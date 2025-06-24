use self::formation::{Formation, FormationMaker};
use crate::components::{Enemy, FromEnemy, Laser, Movable, SpriteSize, Velocity};
use crate::{
    BASE_SPEED, ENEMY_LASER_SIZE, ENEMY_MAX, ENEMY_SIZE, EnemyCount, GameTextures, SPRITE_SCALE,
    WinSize,
};

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use rand::{Rng, thread_rng};
use std::{f32::consts::PI, time::Duration};

mod formation;

/// 敌人插件 - 管理游戏中所有敌人相关的系统和资源
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        // 初始化编队生成器资源
        app.insert_resource(FormationMaker::default())
            // 每秒运行一次敌人生成系统
            .add_systems(
                Update,
                enemy_spawn_system.run_if(on_timer(Duration::from_secs(1))),
            )
            // 满足开火条件时运行敌人开火系统
            .add_systems(Update, enemy_fire_system.run_if(enemy_fire_criteria))
            // 每帧运行敌人移动系统
            .add_systems(Update, enemy_movement_system);
    }
}

/// 敌人生成系统 - 控制敌人的生成逻辑
fn enemy_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    mut enemy_count: ResMut<EnemyCount>,
    mut formation_maker: ResMut<FormationMaker>,
    win_size: Res<WinSize>,
) {
    // 确保敌人数量不超过最大值
    if enemy_count.0 < ENEMY_MAX {
        // 从编队生成器获取编队参数
        let formation = formation_maker.make(&win_size);
        let (x, y) = formation.start;

        // 生成敌人实体
        commands
            .spawn((
                // 设置敌人精灵
                Sprite::from_image(game_textures.enemy.clone()),
                Transform {
                    translation: Vec3::new(x, y, 10.), // Z轴设为10，确保显示在背景上方
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                    ..Default::default()
                },
            ))
            .insert(Enemy) // 标记为敌人实体
            .insert(formation) // 添加编队组件控制移动
            .insert(SpriteSize::from(ENEMY_SIZE)); // 设置精灵大小

        enemy_count.0 += 1; // 更新敌人计数器
    }
}

/// 敌人开火条件 - 随机决定是否开火
fn enemy_fire_criteria() -> bool {
    // 约每60帧有1次机会开火(约1秒1次)
    thread_rng().gen_bool(1. / 60.)
}

/// 敌人开火系统 - 控制敌人发射激光
fn enemy_fire_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    // 遍历所有敌人
    for &tf in enemy_query.iter() {
        let (x, y) = (tf.translation.x, tf.translation.y);

        // 生成敌人激光
        commands
            .spawn((
                Sprite::from_image(game_textures.enemy_laser.clone()),
                Transform {
                    translation: Vec3::new(x, y - 15., 0.), // 激光初始位置
                    rotation: Quat::from_rotation_x(PI),    // 旋转180度，使激光朝下
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                },
            ))
            .insert(Laser) // 标记为激光实体
            .insert(SpriteSize::from(ENEMY_LASER_SIZE)) // 设置激光大小
            .insert(FromEnemy) // 标记为敌人发射的激光
            .insert(Movable { auto_despawn: true }) // 可移动且超出屏幕自动销毁
            .insert(Velocity { x: 0., y: -1. }); // 设置向下的速度
    }
}

/// 敌人移动系统 - 控制敌人按照编队参数移动
fn enemy_movement_system(
    time: Res<Time>,
    win_size: Res<WinSize>,
    mut query: Query<(&mut Transform, &mut Formation), With<Enemy>>,
) {
    let delta = time.delta_secs(); // 获取每帧时间间隔

    for (mut transform, mut formation) in &mut query {
        // 1. 更新编队参数（每0.5秒随机调整一次）
        formation.change_timer += delta;

        // 每0.5秒随机改变移动参数，使编队动态变化
        if formation.change_timer > 0.5 {
            let mut rng = thread_rng();
            formation.pivot_delta = (rng.gen_range(-20.0..20.0), rng.gen_range(-20.0..20.0));
            formation.radius_delta = (rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0));
            formation.speed_delta = rng.gen_range(-10.0..10.0);
            formation.change_timer = 0.0;
        }

        // 应用参数变化
        formation.pivot.0 += formation.pivot_delta.0 * delta;
        formation.pivot.1 += formation.pivot_delta.1 * delta;
        formation.radius.0 += formation.radius_delta.0 * delta;
        formation.radius.1 += formation.radius_delta.1 * delta;
        formation.speed += formation.speed_delta * delta;

        // 限制参数在合理范围内，防止异常
        let w_span = win_size.w / 4.;
        let h_span = win_size.h / 3. - 50.;
        formation.pivot.0 = formation.pivot.0.clamp(-w_span, w_span);
        formation.pivot.1 = formation.pivot.1.clamp(0.0, h_span);
        formation.radius.0 = formation.radius.0.clamp(50.0, 200.0);
        formation.radius.1 = formation.radius.1.clamp(50.0, 150.0);
        formation.speed = formation.speed.clamp(BASE_SPEED * 0.5, BASE_SPEED * 1.5);

        // 2. 计算敌人位置（沿椭圆轨迹移动）
        let (x_org, y_org) = (transform.translation.x, transform.translation.y);
        let max_distance = delta * formation.speed; // 每帧最大移动距离

        // 决定移动方向（根据起始位置确定顺时针/逆时针）
        let dir: f32 = if formation.start.0 < 0. { 1. } else { -1. };
        let (x_pivot, y_pivot) = formation.pivot;
        let (x_radius, y_radius) = formation.radius;

        // 计算下一个角度（基于时间和速度）
        let angle =
            formation.angle + dir * formation.speed * delta / (x_radius.min(y_radius) * PI / 2.);

        // 计算目标位置（椭圆轨迹上的点）
        let x_dst = x_radius * angle.cos() + x_pivot;
        let y_dst = y_radius * angle.sin() + y_pivot;

        // 计算当前位置与目标位置的距离
        let dx = x_org - x_dst;
        let dy = y_org - y_dst;
        let distance = (dx * dx + dy * dy).sqrt();
        let distance_ratio = if distance == 0. {
            0.
        } else {
            max_distance / distance
        };

        // 计算最终位置（平滑过渡到目标位置）
        let x = x_org - dx * distance_ratio;
        let x = if dx > 0. { x.max(x_dst) } else { x.min(x_dst) };
        let y = y_org - dy * distance_ratio;
        let y = if dy > 0. { y.max(y_dst) } else { y.min(y_dst) };

        // 只有当敌人接近椭圆轨迹时才更新角度，确保平滑过渡
        if distance < max_distance * formation.speed / 20. {
            formation.angle = angle;
        }

        // 更新敌人位置
        transform.translation.x = x;
        transform.translation.y = y;
    }
}
