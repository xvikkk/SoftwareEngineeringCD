use crate::{BASE_SPEED, FORMATION_MEMBERS_MAX, WinSize};
use bevy::prelude::{Component, Resource};
use rand::{Rng, thread_rng};
use std::f32::consts::PI;

/// 组件 - 敌人编队（每个敌人都有）
/// 控制敌人在编队中的运动参数和轨迹
#[derive(Clone, Component)]
pub struct Formation {
    pub start: (f32, f32),        // 起始位置坐标(x,y)
    pub radius: (f32, f32),       // 椭圆轨迹的半径(x轴半径,y轴半径)
    pub pivot: (f32, f32),        // 椭圆轨迹的中心点坐标
    pub speed: f32,               // 移动速度
    pub angle: f32,               // 每帧变化的角度
    pub change_timer: f32,        // 参数变化计时器
    pub pivot_delta: (f32, f32),  // 中心点变化速度
    pub radius_delta: (f32, f32), // 半径变化速度
    pub speed_delta: f32,         // 速度变化率
}

/// 资源 - 编队生成器
/// 负责创建和管理敌人编队模板
#[derive(Default, Resource)]
pub struct FormationMaker {
    current_template: Option<Formation>, // 当前使用的编队模板
    current_members: u32,                // 当前编队中的敌人数量
}

/// 编队工厂实现
impl FormationMaker {
    /// 创建一个新的编队或使用现有模板
    ///
    /// 参数:
    /// - win_size: 窗口尺寸，用于计算编队参数
    ///
    /// 返回:
    /// 一个新的Formation实例，用于控制敌人移动
    pub fn make(&mut self, win_size: &WinSize) -> Formation {
        match (
            &self.current_template,
            self.current_members >= FORMATION_MEMBERS_MAX,
        ) {
            // 如果有当前模板且未达到最大成员数，则克隆模板
            (Some(tmpl), false) => {
                self.current_members += 1;
                tmpl.clone()
            }
            // 如果是第一个编队或前一个编队已满，则创建新编队
            (None, _) | (_, true) => {
                let mut rng = thread_rng();

                // 计算起始x/y坐标
                // 从屏幕左侧或右侧随机位置生成
                let w_span = win_size.w / 2. + 100.;
                let h_span = win_size.h / 2. + 100.;
                let x = if rng.gen_bool(0.5) { w_span } else { -w_span };
                let y = rng.gen_range(-h_span..h_span);
                let start = (x, y);

                // 计算椭圆轨迹中心点x/y坐标
                let w_span = win_size.w / 4.;
                let h_span = win_size.h / 3. - 50.;
                let pivot = (rng.gen_range(-w_span..w_span), rng.gen_range(0.0..h_span));

                // 计算椭圆轨迹半径
                let radius = (rng.gen_range(80.0..150.), 100.);

                // 计算起始角度（朝向中心点）
                let angle = (y - pivot.1).atan2(x - pivot.0);

                // 速度（目前固定）
                let speed = BASE_SPEED;

                // 随机生成参数变化速度
                // 这些参数将用于后续动态调整编队
                let pivot_delta = (rng.gen_range(-20.0..20.0), rng.gen_range(-20.0..20.0));
                let radius_delta = (rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0));
                let speed_delta = rng.gen_range(-10.0..10.0);

                // 创建编队实例
                let formation = Formation {
                    start,
                    radius,
                    pivot,
                    speed,
                    angle,
                    change_timer: 0.0,
                    pivot_delta,
                    radius_delta,
                    speed_delta,
                };

                // 存储为模板，以便后续敌人复用相同的编队参数
                self.current_template = Some(formation.clone());
                // 重置成员计数为1
                self.current_members = 1;

                formation
            }
        }
    }
}
