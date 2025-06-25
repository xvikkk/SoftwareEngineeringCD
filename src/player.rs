use crate::components::{FromPlayer, Laser, Movable, Player, SpriteSize, Velocity};
use crate::{
    GameTextures, PLAYER_LASER_SIZE, PLAYER_RESPAWN_DELAY, PLAYER_SIZE, PlayerState, SPRITE_SCALE,
    WinSize,
};

// 玩家移动速度常量
pub const PLAYER_SPEED: f32 = 1.0;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

/// 无敌状态组件
#[derive(Component)]
pub struct Invincible {
    pub timer: Timer,
}

/// 玩家系统插件 - 管理玩家的生成、移动和射击逻辑
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // 初始化玩家状态资源
        app.insert_resource(PlayerState::default())
            // 每0.5秒检查一次玩家重生条件
            .add_systems(
                Update,
                player_spawn_system.run_if(on_timer(Duration::from_secs_f32(0.5))),
            )
            // 处理玩家键盘输入事件
            .add_systems(Update, player_keyboard_event_system)
            // 处理玩家移动和边界检查
            .add_systems(
                Update,
                player_movement_system.after(player_keyboard_event_system),
            )
            // 处理玩家射击逻辑
            .add_systems(Update, player_fire_system)
            // 新增无敌状态计时器系统
            .add_systems(Update, invincible_timer_system);
    }
}

/// 无敌状态计时器系统
fn invincible_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Invincible)>,
) {
    for (entity, mut invincible) in query.iter_mut() {
        invincible.timer.tick(time.delta());
        if invincible.timer.finished() {
            commands.entity(entity).remove::<Invincible>();
        }
    }
}

/// 玩家移动系统 - 控制玩家的移动逻辑
fn player_movement_system(
    time: Res<Time>,
    win_size: Res<WinSize>,
    mut query: Query<(&Velocity, &SpriteSize, &mut Transform), With<Player>>,
) {
    if let Ok((velocity, sprite_size, mut transform)) = query.get_single_mut() {
        // 计算玩家实际尺寸（缩放后）
        let scaled_width = sprite_size.0.x * SPRITE_SCALE;
        let scaled_height = sprite_size.0.y * SPRITE_SCALE;

        // 计算边界（玩家不能超出边界）
        let min_x = -win_size.w / 2. + scaled_width / 2.;
        let max_x = win_size.w / 2. - scaled_width / 2.;
        let min_y = -win_size.h / 2. + scaled_height / 2.;
        let max_y = win_size.h / 2. - scaled_height / 2.;

        // 根据速度和时间步长更新位置
        let delta = time.delta().as_secs_f32();
        let mut new_x = transform.translation.x + velocity.x * delta;
        let mut new_y = transform.translation.y + velocity.y * delta;

        // 限制在边界内
        new_x = new_x.clamp(min_x, max_x);
        new_y = new_y.clamp(min_y, max_y);

        // 更新位置
        transform.translation.x = new_x;
        transform.translation.y = new_y;
    }
}

/// 玩家重生系统 - 控制玩家的生成时机
fn player_spawn_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    game_textures: Res<GameTextures>,
    win_size: Res<WinSize>,
) {
    let now = time.elapsed_secs_f64(); // 当前游戏时间
    let last_shot = player_state.last_shot; // 玩家最后一次死亡时间

    // 条件：玩家未存活，且重生延迟已过（或首次生成）
    if !player_state.on && (last_shot == -1. || now > last_shot + PLAYER_RESPAWN_DELAY) {
        // 计算玩家生成位置（屏幕底部中央）
        let bottom = -win_size.h / 2.;
        commands
            .spawn((
                // 玩家精灵
                Sprite::from_image(game_textures.player.clone()),
                Transform {
                    // 位置：底部中央偏上，Z轴设为10确保显示在背景上方
                    translation: Vec3::new(
                        0.,
                        bottom + PLAYER_SIZE.1 / 2. * SPRITE_SCALE + 5.,
                        10.,
                    ),
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.), // 精灵缩放
                    ..Default::default()
                },
            ))
            .insert(Player) // 标记为玩家实体
            .insert(SpriteSize::from(PLAYER_SIZE)) // 设置精灵尺寸
            .insert(Movable {
                auto_despawn: false,
            }) // 玩家不会自动销毁
            .insert(Velocity { x: 0., y: 0. }) // 初始速度为0
            .insert(Invincible {
                timer: Timer::from_seconds(2.0, TimerMode::Once), // 2秒无敌状态
            }); // 添加无敌组件

        player_state.spawned(); // 标记玩家已重生
    }
}

/// 玩家射击系统 - 处理空格键发射激光的逻辑
fn player_fire_system(
    mut commands: Commands,
    kb: Res<ButtonInput<KeyCode>>,          // 键盘输入资源
    game_textures: Res<GameTextures>,       // 游戏纹理资源
    query: Query<&Transform, With<Player>>, // 玩家位置查询
) {
    // 获取玩家位置（假设游戏中只有一个玩家）
    if let Ok(player_tf) = query.get_single() {
        // 检测空格键是否刚按下
        if kb.just_pressed(KeyCode::Space) {
            let (x, y) = (player_tf.translation.x, player_tf.translation.y);
            // 计算激光发射的水平偏移量（从玩家两侧发射）
            let x_offset = PLAYER_SIZE.0 / 2. * SPRITE_SCALE - 5.;

            // 封装激光生成逻辑为闭包
            let mut spawn_laser = |x_offset: f32| {
                commands
                    .spawn((
                        // 玩家激光精灵
                        Sprite::from_image(game_textures.player_laser.clone()),
                        Transform {
                            // 位置：玩家上方两侧
                            translation: Vec3::new(x + x_offset, y + 15., 0.),
                            scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                            ..Default::default()
                        },
                    ))
                    .insert(Laser) // 标记为激光实体
                    .insert(FromPlayer) // 标记为玩家发射的激光
                    .insert(SpriteSize::from(PLAYER_LASER_SIZE)) // 设置激光尺寸
                    .insert(Movable { auto_despawn: true }) // 激光超出屏幕自动销毁
                    .insert(Velocity { x: 0., y: 1. }); // 激光向上移动的速度
            };

            // 从玩家左右两侧各发射一束激光
            spawn_laser(x_offset);
            spawn_laser(-x_offset);
        }
    }
}

/// 玩家键盘事件系统 - 处理方向键控制玩家移动
fn player_keyboard_event_system(
    kb: Res<ButtonInput<KeyCode>>,                 // 键盘输入资源
    mut query: Query<&mut Velocity, With<Player>>, // 玩家速度组件查询
) {
    // 获取玩家速度组件（假设游戏中只有一个玩家）
    if let Ok(mut velocity) = query.get_single_mut() {
        // 初始化速度向量
        let mut input_velocity = Vec2::new(0., 0.);

        // 处理水平输入
        if kb.pressed(KeyCode::ArrowLeft) {
            input_velocity.x -= 1.0;
        }
        if kb.pressed(KeyCode::ArrowRight) {
            input_velocity.x += 1.0;
        }

        // 处理垂直输入
        if kb.pressed(KeyCode::ArrowUp) {
            input_velocity.y += 1.0;
        }
        if kb.pressed(KeyCode::ArrowDown) {
            input_velocity.y -= 1.0;
        }

        // 归一化速度向量以确保对角线移动速度一致
        if input_velocity.length_squared() > 0.0 {
            input_velocity = input_velocity.normalize() * PLAYER_SPEED;
        }

        // 更新速度组件
        velocity.x = input_velocity.x;
        velocity.y = input_velocity.y;
    }
}
