//! 健身贴士的数据层：三分法（推 / 拉 / 腿）训练计划与每周安排。
//! 计划按星期几存一个 0~3 的编号：0 休息、1 推日、2 拉日、3 腿日。
//! 推日练胸肩（俯卧撑、推举），拉日练背与后肩（引体、划船），腿日练下肢。

/// 每周计划默认值：周一到周三分别推、拉、腿，其余休息。
pub const DEFAULT_PLAN: [u8; 7] = [PUSH, PULL, LEGS, REST, REST, REST, REST];

pub const REST: u8 = 0;
pub const PUSH: u8 = 1;
pub const PULL: u8 = 2;
pub const LEGS: u8 = 3;

pub const PART_NAMES: [&str; 4] = ["休息", "推日", "拉日", "腿日"];

/// 星期几的名称，周一开始，与计划的下标对应。
pub const WEEKDAYS: [&str; 7] = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];

/// 一个动作：名称、组次或时长、一句要点、示意图文件名（assets/fitness/ 下）。
#[derive(Clone, Copy)]
pub struct Move {
    pub name: &'static str,
    pub dose: &'static str,
    pub tip: &'static str,
    pub icon: &'static str,
}

pub struct Part {
    pub warmup: &'static [Move],
    pub training: &'static [Move],
}

const PUSH_PART: Part = Part {
    warmup: &[
        Move { name: "开合跳", dose: "30 秒", tip: "落地屈膝缓冲，心率慢慢提起来", icon: "chest-jack" },
        Move { name: "肩绕环", dose: "前后各 10 圈", tip: "幅度由小到大，把肩带活动开", icon: "arm-circle" },
        Move { name: "跪姿俯卧撑", dose: "8 次", tip: "慢下快起，唤醒胸和手臂", icon: "chest-knee-pushup" },
    ],
    training: &[
        Move { name: "标准俯卧撑", dose: "3 组 × 12", tip: "身体一条直线，胸贴近地再推起", icon: "chest-pushup" },
        Move { name: "上斜俯卧撑", dose: "3 组 × 10", tip: "手撑稳固椅面，侧重胸上部", icon: "chest-incline" },
        Move { name: "哑铃推举", dose: "3 组 × 10", tip: "核心收紧，把哑铃推向头顶", icon: "sh-press" },
        Move { name: "哑铃飞鸟", dose: "3 组 × 12", tip: "肘部微屈，像抱住一棵大树", icon: "chest-fly" },
    ],
};

const PULL_PART: Part = Part {
    warmup: &[
        Move { name: "猫式伸展", dose: "10 次", tip: "呼气弓背，吸气塌腰", icon: "back-cat" },
        Move { name: "超人式", dose: "10 次", tip: "四肢离地停 2 秒，收紧下背", icon: "back-superman" },
        Move { name: "弹力带划船", dose: "15 次", tip: "肩胛骨先收，再拉手肘", icon: "back-band-row" },
    ],
    training: &[
        Move { name: "引体向上", dose: "3 组 × 力竭", tip: "下巴过杠，下放要慢", icon: "back-pullup" },
        Move { name: "俯身划船", dose: "3 组 × 12", tip: "背部平直，把杠拉向小腹", icon: "back-bar-row" },
        Move { name: "单臂哑铃划船", dose: "3 组 × 10", tip: "肘贴身体向后，背带动手臂", icon: "back-one-row" },
        Move { name: "反向飞鸟", dose: "3 组 × 12", tip: "俯身 45°，感受后肩收缩", icon: "sh-reverse" },
    ],
};

const LEGS_PART: Part = Part {
    warmup: &[
        Move { name: "高抬腿", dose: "30 秒", tip: "大腿抬平，前脚掌落地", icon: "leg-high-knee" },
        Move { name: "弓步走", dose: "10 步", tip: "后膝接近地面再起身", icon: "leg-lunge-walk" },
        Move { name: "徒手深蹲", dose: "15 次", tip: "髋往后坐，膝盖对脚尖", icon: "leg-squat-bw" },
    ],
    training: &[
        Move { name: "深蹲", dose: "4 组 × 15", tip: "髋部后坐，大腿平行地面", icon: "leg-squat" },
        Move { name: "弓步蹲", dose: "3 组 × 10 / 腿", tip: "重心在前脚跟，膝不内扣", icon: "leg-lunge" },
        Move { name: "臀桥", dose: "3 组 × 15", tip: "顶点夹紧臀部停 2 秒", icon: "leg-bridge" },
        Move { name: "提踵", dose: "3 组 × 20", tip: "踮到最高点，缓慢下放", icon: "leg-calf" },
    ],
};

const REST_PART: Part = Part {
    warmup: &[],
    training: &[
        Move { name: "静态拉伸", dose: "10 分钟", tip: "每个部位拉 30 秒，别弹震", icon: "rest-stretch" },
        Move { name: "散步", dose: "20 分钟", tip: "强度刚好还能聊天最合适", icon: "rest-walk" },
        Move { name: "泡沫轴放松", dose: "10 分钟", tip: "滚到酸点停 20 秒再继续", icon: "rest-foam" },
    ],
};

/// 部位编号对应的训练清单；休息日给恢复建议。
pub fn plan(part: u8) -> Option<&'static Part> {
    match part {
        PUSH => Some(&PUSH_PART),
        PULL => Some(&PULL_PART),
        LEGS => Some(&LEGS_PART),
        REST => Some(&REST_PART),
        _ => None,
    }
}

/// 今天是星期几（周一开始，0..6）。
pub fn weekday() -> usize {
    use chrono::Datelike;
    chrono::Local::now().weekday().num_days_from_monday() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_part_has_a_plan_with_moves_and_icons() {
        for (part, name) in PART_NAMES.iter().enumerate() {
            let plan = plan(part as u8).unwrap_or_else(|| panic!("{name} 缺少计划"));
            let total = plan.warmup.len() + plan.training.len();
            assert!(total >= 3, "{name} 的动作太少");
            for mv in plan.warmup.iter().chain(plan.training.iter()) {
                assert!(!mv.name.is_empty() && !mv.dose.is_empty() && !mv.tip.is_empty(), "{name} 动作信息不完整");
                assert!(mv.icon.ends_with(".svg") || mv.icon.contains('-'), "图标名应是文件名：{}", mv.icon);
            }
        }
    }

    #[test]
    fn rest_day_has_only_recovery_and_training_parts_have_both() {
        let rest = plan(REST).unwrap();
        assert!(rest.warmup.is_empty() && rest.training.len() == 3);
        for part in [PUSH, PULL, LEGS] {
            let plan = plan(part).unwrap();
            assert_eq!(plan.warmup.len(), 3, "{} 热身应 3 个", PART_NAMES[part as usize]);
            assert_eq!(plan.training.len(), 4, "{} 训练应 4 个", PART_NAMES[part as usize]);
        }
    }

    #[test]
    fn default_plan_is_a_three_day_split() {
        assert_eq!(DEFAULT_PLAN, [PUSH, PULL, LEGS, REST, REST, REST, REST]);
        for part in DEFAULT_PLAN {
            assert!(plan(part).is_some(), "默认计划里的部位 {} 不存在", part);
        }
    }

    #[test]
    fn weekday_names_match_plan_index() {
        assert_eq!(WEEKDAYS.len(), 7);
        assert_eq!(PART_NAMES.len(), 4);
    }
}
