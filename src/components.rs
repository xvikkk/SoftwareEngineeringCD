use bevy::math::{Vec2, Vec3};
use bevy::prelude::Component;
use bevy::time::{Timer, TimerMode};

// region:    --- 通用组件
/// 速度组件 - 控制实体的移动速度
#[derive(Component)]
pub struct Velocity {
    pub x: f32, // X轴方向速度
    pub y: f32, // Y轴方向速度
}

/// 可移动组件 - 标记实体可以移动并控制自动销毁行为
#[derive(Component)]
pub struct Movable {
    pub auto_despawn: bool, // 是否超出屏幕后自动销毁
}

/// 激光组件 - 标记实体为激光
#[derive(Component)]
pub struct Laser;

/// 精灵尺寸组件 - 存储精灵的大小
#[derive(Component)]
pub struct SpriteSize(pub Vec2);

/// 从元组(f32, f32)转换为SpriteSize的实现
impl From<(f32, f32)> for SpriteSize {
    fn from(val: (f32, f32)) -> Self {
        SpriteSize(Vec2::new(val.0, val.1))
    }
}
// endregion: --- 通用组件

// region:    --- 玩家相关组件
/// 玩家组件 - 标记玩家实体
#[derive(Component)]
pub struct Player;

/// 玩家来源组件 - 标记实体来自玩家(如玩家发射的激光)
#[derive(Component)]
pub struct FromPlayer;
// endregion: --- 玩家相关组件

// region:    --- 敌人相关组件
/// 敌人组件 - 标记敌人实体
#[derive(Component)]
pub struct Enemy;

/// 敌人来源组件 - 标记实体来自敌人(如敌人发射的激光)
#[derive(Component)]
pub struct FromEnemy;
// endregion: --- 敌人相关组件

// region:    --- 爆炸效果相关组件
/// 爆炸组件 - 标记爆炸实体
#[derive(Component)]
pub struct Explosion;

/// 待生成爆炸组件 - 存储爆炸生成位置
#[derive(Component)]
pub struct ExplosionToSpawn(pub Vec3); // 爆炸位置

/// 爆炸计时器组件 - 控制爆炸动画的播放速度
#[derive(Component)]
pub struct ExplosionTimer(pub Timer);

/// 爆炸计时器默认实现 - 设置为每0.05秒触发一次的重复计时器
impl Default for ExplosionTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.05, TimerMode::Repeating))
    }
}
// endregion: --- 爆炸效果相关组件
