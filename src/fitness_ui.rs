//! 健身贴士页与设置：按每周计划显示今天的部位、热身与训练动作。
//! 页面与日程页同高（CALENDAR_H），主体放在滚动区里，动作多时上下滚动查看。
use crate::app::{App, Cmd, Env};
use crate::fitness;
use crate::paint::{Card, H};
use crate::theme::{self, W};
use eframe::egui::{self, Color32};

/// 部位强调色：推珊瑚红、拉天蓝、腿绿、休息琥珀。
pub(crate) fn accent(part: u8) -> Color32 {
    match part {
        fitness::PUSH => Color32::from_rgb(255, 138, 122),
        fitness::PULL => Color32::from_rgb(111, 182, 255),
        fitness::LEGS => Color32::from_rgb(99, 212, 148),
        _ => Color32::from_rgb(255, 196, 103),
    }
}

/// 动作示意图：内置 SVG 火柴人，按显示尺寸栅格化。
fn move_icon(c: &Card, x: f32, y: f32, size: f32, icon: &str) {
    let source = match icon {
        "chest-jack" => egui::include_image!("../assets/fitness/chest-jack.svg"),
        "chest-knee-pushup" => egui::include_image!("../assets/fitness/chest-knee-pushup.svg"),
        "chest-pushup" => egui::include_image!("../assets/fitness/chest-pushup.svg"),
        "chest-incline" => egui::include_image!("../assets/fitness/chest-incline.svg"),
        "chest-fly" => egui::include_image!("../assets/fitness/chest-fly.svg"),
        "back-cat" => egui::include_image!("../assets/fitness/back-cat.svg"),
        "back-superman" => egui::include_image!("../assets/fitness/back-superman.svg"),
        "back-band-row" => egui::include_image!("../assets/fitness/back-band-row.svg"),
        "back-pullup" => egui::include_image!("../assets/fitness/back-pullup.svg"),
        "back-one-row" => egui::include_image!("../assets/fitness/back-one-row.svg"),
        "back-bar-row" => egui::include_image!("../assets/fitness/back-bar-row.svg"),
        "arm-circle" => egui::include_image!("../assets/fitness/arm-circle.svg"),
        "sh-press" => egui::include_image!("../assets/fitness/sh-press.svg"),
        "sh-reverse" => egui::include_image!("../assets/fitness/sh-reverse.svg"),
        "leg-high-knee" => egui::include_image!("../assets/fitness/leg-high-knee.svg"),
        "leg-lunge-walk" => egui::include_image!("../assets/fitness/leg-lunge-walk.svg"),
        "leg-squat-bw" => egui::include_image!("../assets/fitness/leg-squat-bw.svg"),
        "leg-squat" => egui::include_image!("../assets/fitness/leg-squat.svg"),
        "leg-lunge" => egui::include_image!("../assets/fitness/leg-lunge.svg"),
        "leg-bridge" => egui::include_image!("../assets/fitness/leg-bridge.svg"),
        "leg-calf" => egui::include_image!("../assets/fitness/leg-calf.svg"),
        "rest-stretch" => egui::include_image!("../assets/fitness/rest-stretch.svg"),
        "rest-walk" => egui::include_image!("../assets/fitness/rest-walk.svg"),
        "rest-foam" => egui::include_image!("../assets/fitness/rest-foam.svg"),
        _ => return,
    };
    let rect = c.rect(x, y, size, size);
    let egui::ImageSource::Bytes { uri, bytes } = source else { return };
    let pixels = (size * c.ui.ctx().pixels_per_point()).ceil() as u32;
    let image = egui::Image::from_bytes(format!("{uri}/{pixels}.svg"), bytes);
    if let Ok(egui::load::TexturePoll::Ready { texture }) = image.fit_to_exact_size(rect.size()).load_for_size(c.ui.ctx(), rect.size()) {
        c.painter.image(texture.id, rect, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)), Color32::WHITE);
    }
}

/// 滚动区里的一行动作：示意图 + 名称 + 组次（右对齐强调色）+ 要点。
fn move_row(ui: &mut egui::Ui, mv: &fitness::Move, color: Color32, width: f32, s: f32, live: bool) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width * s, 38.0 * s), egui::Sense::hover());
    let mut row = Card::new(ui, rect.min, ui.clip_rect());
    row.s = s;
    row.live = live;
    move_icon(&row, 6.0, 4.0, 28.0, mv.icon);
    row.text(44.0, 14.0, &row.fit(mv.name, 11.0, W::Semibold, width - 128.0), 11.0, W::Semibold, theme::WHITE, H::Left);
    row.text(width - 14.0, 14.0, mv.dose, 10.0, W::Regular, color, H::Right);
    row.text(44.0, 29.0, &row.fit(mv.tip, 8.5, W::Regular, width - 58.0), 8.5, W::Regular, theme::DIM, H::Left);
}

/// 滚动区里的分组标题。
fn section_label(ui: &mut egui::Ui, text: &str, color: Color32, width: f32, s: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width * s, 15.0 * s), egui::Sense::hover());
    let mut row = Card::new(ui, rect.min, ui.clip_rect());
    row.s = s;
    row.text(2.0, 11.0, text, 9.5, W::Semibold, color, H::Left);
}

impl App {
    /// 今天计划练什么部位（星期几 → 每周计划）。
    pub(crate) fn fitness_today(&self) -> (usize, u8) {
        let day = fitness::weekday();
        (day, self.settings.fitness_plan[day])
    }

    pub(crate) fn page_fitness(&mut self, c: &Card, _env: &Env, _cmds: &mut Vec<Cmd>) {
        let (day, part) = self.fitness_today();
        let Some(plan) = fitness::plan(part) else { return };
        let color = accent(part);

        // 顶部：今天周几练什么；右侧预告明天。
        c.circle(24.0, 40.0, 3.0, color);
        c.text(34.0, 44.0, &format!("{} · {}", fitness::WEEKDAYS[day], fitness::PART_NAMES[part as usize]), 13.0, W::Semibold, theme::WHITE, H::Left);
        let tomorrow = self.settings.fitness_plan[(day + 1) % 7];
        c.text(399.0, 44.0, &format!("明天 · {}", fitness::PART_NAMES[tomorrow as usize]), 9.5, W::Regular, theme::DIM, H::Right);

        // 主体滚动区：与日程页同高，左右留出翻页箭头的位置（照邮件列表的做法：子 Ui + ScrollArea）。
        let (left, top, width, height) = (36.0, 52.0, 347.0, 148.0);
        let bounds = c.rect(left, top, width, height);
        let mut content = egui::Ui::new(c.ui.ctx().clone(), egui::Id::new(("fitness-scroll", c.live)),
            egui::UiBuilder::new().max_rect(bounds).layer_id(c.painter.layer_id()).style(c.ui.style().clone()));
        content.set_clip_rect(bounds.intersect(c.painter.clip_rect()));
        content.set_opacity(c.painter.opacity());
        if !c.live { content.disable(); }
        content.spacing_mut().item_spacing.y = 0.0;
        content.spacing_mut().scroll.floating = false;
        content.spacing_mut().scroll.foreground_color = false;
        content.spacing_mut().scroll.bar_width = 3.0 * c.s;
        content.visuals_mut().widgets.inactive.bg_fill = Color32::from_white_alpha(30);
        content.visuals_mut().widgets.hovered.bg_fill = Color32::from_white_alpha(64);
        content.visuals_mut().widgets.active.bg_fill = Color32::from_white_alpha(100);
        let (s, live) = (c.s, c.live);
        egui::ScrollArea::vertical().id_salt("fitness-scroll").max_height(height * s)
            .auto_shrink([false, false]).scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(&mut content, |ui| {
                if !plan.warmup.is_empty() {
                    section_label(ui, "热身", color, width, s);
                    let chips = plan.warmup.len() as f32;
                    let chip_w = (width - 16.0 - (chips - 1.0) * 8.0) / chips;
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(width * s, 38.0 * s), egui::Sense::hover());
                    let mut row = Card::new(ui, rect.min, ui.clip_rect());
                    row.s = s;
                    row.live = live;
                    for (index, mv) in plan.warmup.iter().enumerate() {
                        let x = index as f32 * (chip_w + 8.0);
                        row.fill(x, 0.0, chip_w, 36.0, 8.0, Color32::from_white_alpha(10));
                        move_icon(&row, x + 8.0, 7.0, 22.0, mv.icon);
                        row.text(x + 36.0, 14.0, &row.fit(mv.name, 10.0, W::Semibold, chip_w - 42.0), 10.0, W::Semibold, theme::WHITE, H::Left);
                        row.text(x + 36.0, 27.0, &row.fit(mv.dose, 8.5, W::Regular, chip_w - 42.0), 8.5, W::Regular, theme::DIM, H::Left);
                    }
                    ui.allocate_exact_size(egui::vec2(width * s, 6.0 * s), egui::Sense::hover());
                }
                section_label(ui, if part == fitness::REST { "恢复建议" } else { "训练动作" }, color, width, s);
                for mv in plan.training.iter() {
                    move_row(ui, mv, color, width, s, live);
                }
                ui.allocate_exact_size(egui::vec2(width * s, 4.0 * s), egui::Sense::hover());
                let (rect, _) = ui.allocate_exact_size(egui::vec2(width * s, 14.0 * s), egui::Sense::hover());
                let mut row = Card::new(ui, rect.min, ui.clip_rect());
                row.s = s;
                let hint = if part == fitness::REST {
                    "今天是休息日，训练日练完也可以做这些放松。"
                } else {
                    "组间休息 60~90 秒；疼痛就停，酸痛正常。"
                };
                row.text(2.0, 11.0, hint, 8.5, W::Regular, theme::DIM, H::Left);
            });
    }

    pub(crate) fn fitness_settings(&mut self, ui: &mut egui::Ui) {
        use crate::extras::{settings_card, settings_control_row, settings_hint, settings_label};
        let today = fitness::weekday();
        settings_label(ui, "今日");
        settings_card(ui, |ui| {
            let part = self.settings.fitness_plan[today];
            let text = if part == fitness::REST {
                format!("今天{} · 休息", fitness::WEEKDAYS[today])
            } else {
                format!("今天{} · {}", fitness::WEEKDAYS[today], fitness::PART_NAMES[part as usize])
            };
            ui.label(egui::RichText::new(text).size(13.0));
            settings_hint(ui, "健身页按这里的安排显示当天的热身与训练动作，改完立即生效。");
        });

        settings_label(ui, "每周计划");
        settings_card(ui, |ui| {
            for day in 0..7 {
                settings_control_row(ui, fitness::WEEKDAYS[day], 56.0, |ui| {
                    ui.horizontal(|ui| {
                        for (part, name) in fitness::PART_NAMES.iter().enumerate() {
                            if ui.selectable_label(self.settings.fitness_plan[day] == part as u8, *name).clicked() {
                                self.settings.fitness_plan[day] = part as u8;
                            }
                        }
                    }).response
                });
            }
            settings_hint(ui, "三分法：推日练胸和肩，拉日练背和后肩，腿日练下肢；计划只保存在本机配置里。");
        });
    }
}
