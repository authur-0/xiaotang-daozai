//! 系统跳转不可用时，定位播放器自己的进度控件并向该窗口发送消息。
use std::time::Duration;
use windows::{core::{Interface,VARIANT},Win32::{Foundation::*,System::{Com::*,Threading::*},UI::{Accessibility::*,WindowsAndMessaging::{*,EnumChildWindows}},Graphics::Gdi::ScreenToClient}};
use windows::Win32::UI::HiDpi::{SetThreadDpiAwarenessContext,DPI_AWARENESS_CONTEXT,DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};
use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;

struct DpiContext(DPI_AWARENESS_CONTEXT);
impl Drop for DpiContext{fn drop(&mut self){if !self.0.0.is_null(){unsafe{SetThreadDpiAwarenessContext(self.0);}}}}

fn seconds(text:&str)->Option<f64>{
    let parts=text.trim().split(':').map(str::parse::<u32>).collect::<Result<Vec<_>,_>>().ok()?;
    match parts.as_slice(){[m,s] if *s<60=>Some((m*60+s)as f64),[h,m,s] if *m<60&&*s<60=>Some((h*3600+m*60+s)as f64),_=>None}
}

fn point(rect:RECT,position:f64,duration:f64,inset:f64)->Option<POINT>{
    let width=(rect.right-rect.left)as f64;
    if !position.is_finite() || !duration.is_finite() || duration<=0.0 || position<0.0 || position>duration
        || width<=inset*2.0+10.0 || rect.bottom<=rect.top{return None;}
    Some(POINT{x:((rect.left as f64+inset+(width-2.0*inset)*position/duration).round()as i32).min(rect.right-1),y:rect.top+(rect.bottom-rect.top)/2})
}

unsafe fn matches_player(hwnd:HWND,source:&str)->bool{
    let mut pid=0;GetWindowThreadProcessId(hwnd,Some(&mut pid));
    let Ok(process)=OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION,false,pid)else{return false;};
    let mut name=[0u16;32768];let mut len=name.len()as u32;
    let result=QueryFullProcessImageNameW(process,PROCESS_NAME_WIN32,windows::core::PWSTR(name.as_mut_ptr()),&mut len);
    let _=CloseHandle(process);
    result.is_ok() && String::from_utf16_lossy(&name[..len as usize]).rsplit('\\').next().is_some_and(|name|name.eq_ignore_ascii_case(source))
}

unsafe fn window(source:&str)->Option<HWND>{
    struct Search<'a>{source:&'a str,class:Option<&'a str>,found:Option<HWND>}
    unsafe extern "system" fn visit(hwnd:HWND,param:LPARAM)->BOOL{
        let search=&mut *(param.0 as *mut Search<'_>);
        // Apple Music 按窗口标题找；其余播放器的顶层窗口类名固定（QQ 为 TXGuiFoundation，网易云为 OrpheusBrowserHost）。
        let matched=match search.class {
            Some(class)=>{let mut name=[0u16;80];let n=GetClassNameW(hwnd,&mut name);String::from_utf16_lossy(&name[..n.max(0)as usize])==class}
            None=>{let mut name=[0u16;80];let n=GetWindowTextW(hwnd,&mut name);String::from_utf16_lossy(&name[..n.max(0)as usize])=="Apple Music"}
        };
        if matched && IsWindowVisible(hwnd).as_bool() && GetWindow(hwnd,GW_OWNER).is_err() && GetWindowTextLengthW(hwnd)>0 && matches_player(hwnd,search.source){search.found=Some(hwnd);return BOOL(0);}
        BOOL(1)
    }
    let apple=super::apple_music::is_source(source);
    let class=if apple{None}else if source.eq_ignore_ascii_case("cloudmusic.exe"){Some("OrpheusBrowserHost")}else{Some("TXGuiFoundation")};
    let mut search=Search{source:if apple{"AppleMusic.exe"}else{source},class,found:None};
    let _=EnumWindows(Some(visit),LPARAM(&mut search as *mut _ as isize));search.found
}

unsafe fn qq_bar(ui:&IUIAutomation,root:&IUIAutomationElement,duration:f64)->Option<IUIAutomationElement>{
    let condition=ui.CreatePropertyCondition(UIA_ControlTypePropertyId,&VARIANT::from(UIA_TextControlTypeId.0)).ok()?;
    let texts=root.FindAll(TreeScope_Descendants,&condition).ok()?;
    let walker=ui.ControlViewWalker().ok()?;let all=ui.CreateTrueCondition().ok()?;
    for i in 0..texts.Length().ok()? {
        let text=texts.GetElement(i).ok()?;
        if !seconds(&text.CurrentName().ok()?.to_string()).is_some_and(|v|(v-duration).abs()<1.5){continue;}
        let parent=walker.GetParentElement(&text).ok()?;
        let children=parent.FindAll(TreeScope_Children,&all).ok()?;
        if children.Length().ok()?!=3{continue;}
        let first=children.GetElement(0).ok()?;let last=children.GetElement(2).ok()?;
        if seconds(&first.CurrentName().ok()?.to_string()).is_none() || !ui.CompareElements(&last,&text).ok()?.as_bool(){continue;}
        let bar=children.GetElement(1).ok()?;
        let r=bar.CurrentBoundingRectangle().ok()?;
        let left=first.CurrentBoundingRectangle().ok()?;let right=last.CurrentBoundingRectangle().ok()?;
        if r.left>=left.right && r.right<=right.left && r.bottom-r.top<=60 && r.right-r.left>40{return Some(bar);}
    }
    None
}

/// 网易云 CEF 界面把进度条只暴露在原始视图（RawView）里，Control 视图看不到，
/// 因此不能用 FindFirst，必须用 RawViewWalker 手动遍历。
unsafe fn netease_bar(ui:&IUIAutomation,root:&IUIAutomationElement)->Option<IUIAutomationElement>{
    let walker=ui.RawViewWalker().ok()?;
    unsafe fn find_named(walker:&IUIAutomationTreeWalker,element:&IUIAutomationElement,name:&str,depth:usize)->Option<IUIAutomationElement>{
        if depth>16 {return None;}
        if let Ok(current)=element.CurrentName() {if current.to_string()==name {return Some(element.clone());}}
        let Ok(mut child)=walker.GetFirstChildElement(element)else{return None;};
        let mut count=0;
        loop {
            if let Some(found)=find_named(walker,&child,name,depth+1) {return Some(found);}
            let Ok(sibling)=walker.GetNextSiblingElement(&child)else{break;};
            child=sibling;
            count+=1;
            if count>2000 {break;}
        }
        None
    }
    find_named(&walker,root,"播放进度调节",0)
}

unsafe fn automation()->Option<IUIAutomation>{
    let ui:IUIAutomation=CoCreateInstance(&CUIAutomation8,None,CLSCTX_INPROC_SERVER).ok()?;
    if let Ok(timed)=ui.cast::<IUIAutomation2>(){let _=timed.SetConnectionTimeout(800);let _=timed.SetTransactionTimeout(800);}
    Some(ui)
}
unsafe fn apple_bar(ui:&IUIAutomation,root:&IUIAutomationElement)->Option<IUIAutomationRangeValuePattern>{
    let condition=ui.CreatePropertyCondition(UIA_AutomationIdPropertyId,&VARIANT::from("LCDScrubber")).ok()?;
    root.FindFirst(TreeScope_Descendants,&condition).ok()?.GetCurrentPattern(UIA_RangeValuePatternId).ok()?.cast().ok()
}
fn apple_value(range:&IUIAutomationRangeValuePattern,duration:f64)->Option<(f64,f64)>{
    unsafe {
        let (min,max,value)=(range.CurrentMinimum().ok()?,range.CurrentMaximum().ok()?,range.CurrentValue().ok()?);
        (duration.is_finite() && duration>0.0 && min==0.0 && max.is_finite() && (max-duration).abs()<1.5
            && value.is_finite() && (min..=max).contains(&value)).then_some((value,max))
    }
}
pub(super) fn seek(source:&str,position:f64,duration:f64)->bool{
    let apple=super::apple_music::is_source(source);
    let netease=source.eq_ignore_ascii_case("cloudmusic.exe");
    if !apple && !netease && !source.eq_ignore_ascii_case("qqmusic.exe"){return false;}
    unsafe {
        let hwnd=match window(source){Some(hwnd)=>hwnd,None=>return false};
        // 网易云最小化或隐藏到托盘时，Chromium 会停用无障碍树，「播放进度调节」不再暴露，
        // 跳转必然失败。先无激活地恢复显示（SW_SHOWNOACTIVATE 不抢焦点），结束后还原窗口状态。
        let was_iconic=IsIconic(hwnd).as_bool();
        let was_visible=IsWindowVisible(hwnd).as_bool();
        let hidden=netease && (was_iconic || !was_visible);
        if hidden {ShowWindow(hwnd,SW_SHOWNOACTIVATE);}
        let result=(||->Option<()> {
        let _dpi=DpiContext(SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2));
        if !IsWindowEnabled(hwnd).as_bool(){return None;}
        let ui=automation()?;
        let root=ui.ElementFromHandle(hwnd).ok()?;
        if apple {
            let range=apple_bar(&ui,&root)?;let (_,max)=apple_value(&range,duration)?;
            if range.CurrentIsReadOnly().ok()?.as_bool() || !position.is_finite() || !(0.0..=max).contains(&position){return None;}
            // 直接设置原生滑块；无需移动鼠标、激活窗口或扫描歌词区域。
            return range.SetValue(position).ok();
        }
        // 刚恢复显示或窗口被完全遮挡时，Chromium 重建无障碍树需要一点时间；重试覆盖这个延迟。
        let bar=if netease {
            let mut found=None;
            for _ in 0..5 {
                if let Ok(root)=ui.ElementFromHandle(hwnd) {if let Some(bar)=netease_bar(&ui,&root){found=Some(bar);break;}}
                std::thread::sleep(Duration::from_millis(400));
            }
            found?
        }else{qq_bar(&ui,&root,duration)?};
        let debug=std::env::var_os("DAOZAI_SEEK_DEBUG").is_some();
        if debug {
            let r=bar.CurrentBoundingRectangle().map(|r|format!("({},{}) {}x{}",r.left,r.top,r.right-r.left,r.bottom-r.top)).unwrap_or_default();
            let sink=FindWindowExW(hwnd,None,windows::core::w!("Chrome_RenderWidgetHostHWND"),None);
            eprintln!("[seek-debug] hwnd={hwnd:?} enabled={} bar={r} render={sink:?}",IsWindowEnabled(hwnd).as_bool());
        }
        let target=hwnd;
        // 坐标来自无障碍控件，不依赖窗口位置；不移动鼠标，也不激活播放器。
        let mut p=point(bar.CurrentBoundingRectangle().ok()?,position,duration,5.0)?;
        if !ScreenToClient(target,&mut p).as_bool(){return None;}
        let mut client=RECT::default();GetClientRect(target,&mut client).ok()?;
        if p.x<0 || p.y<0 || (!IsIconic(hwnd).as_bool() && (p.x>=client.right || p.y>=client.bottom)){return None;}
        let lp=LPARAM(((p.y as u32&0xffff)<<16 | (p.x as u32&0xffff))as isize);
        // Chromium 的输入由 Chrome_RenderWidgetHostHWND 子窗口转发；找不到时退回顶层窗口。
        let sink={
            let mut children:Vec<HWND>=Vec::new();
            unsafe extern "system" fn collect(h:HWND,l:LPARAM)->BOOL{unsafe{let v=&mut *(l.0 as *mut Vec<HWND>);v.push(h);BOOL(1)}}
            let _=EnumChildWindows(target,Some(collect),LPARAM(&mut children as *mut _ as isize));
            children.into_iter().find(|child|{
                let mut name=[0u16;64];let n=GetClassNameW(*child,&mut name);
                String::from_utf16_lossy(&name[..n.max(0)as usize])=="Chrome_RenderWidgetHostHWND"
            }).unwrap_or(target)
        };
        if debug {eprintln!("[seek-debug] point=({},{}) sink={sink:?} target={target:?}",p.x,p.y);}
        let send_to=|sink:HWND,msg,buttons|SendMessageTimeoutW(sink,msg,WPARAM(buttons),lp,SMTO_ABORTIFHUNG|SMTO_ERRORONEXIT,500,None).0!=0;
        let send=|msg,buttons|if send_to(sink,msg,buttons){true}else{send_to(target,msg,buttons)};
        if !send(WM_MOUSEMOVE,0){return None;}
        let down=send(WM_LBUTTONDOWN,1);
        let up=send(WM_LBUTTONUP,0);
        if debug {eprintln!("[seek-debug] down={down} up={up}");}
        (down&&up).then_some(())
        })().is_some();
        // 点击完成后还原窗口状态：等 CEF 处理完消息再最小化/隐藏，避免打断消息处理。
        if hidden {
            std::thread::sleep(Duration::from_millis(300));
            ShowWindow(hwnd,if was_iconic {SW_MINIMIZE}else{SW_HIDE});
        }
        result
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test] fn progress_coordinates_follow_control_bounds_and_reject_invalid_input(){
        for left in [-31000,100,2300]{let r=RECT{left,top:500,right:left+260,bottom:524};
            assert_eq!(point(r,0.0,200.0,5.0).unwrap().x,left+5);
            assert_eq!(point(r,100.0,200.0,5.0).unwrap().x,left+130);
            assert_eq!(point(r,200.0,200.0,5.0).unwrap().x,left+255);
            assert_eq!(point(r,200.0,200.0,0.0).unwrap().x,r.right-1);
            for p in [f64::NAN,-1.0,201.0]{assert!(point(r,p,200.0,5.0).is_none());}
            assert!(point(r,0.0,0.0,5.0).is_none());
        }
        assert_eq!(seconds(" 04:41"),Some(281.0));assert_eq!(seconds("01:02:03"),Some(3723.0));
        for text in ["播放","04:99","", "1:2:3:4"]{assert!(seconds(text).is_none());}
    }
}
