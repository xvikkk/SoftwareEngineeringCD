#![allow(unused)] // 探索阶段用来屏蔽未使用警告

use bevy::audio::{AudioPlayer, PlaybackSettings}; // 用于音频播放
use bevy::math::bounding::IntersectsVolume;
use bevy::math::{Vec3Swizzles, bounding::Aabb2d};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use components::{
    Enemy, Explosion, ExplosionTimer, ExplosionToSpawn, FromEnemy, FromPlayer, Laser, Movable,
    Player, SpriteSize, Velocity,
};
use enemy::EnemyPlugin;
use player::PlayerPlugin;
use std::collections::HashSet;

mod components; // 组件模块
mod enemy; // 敌人相关模块
mod player; // 玩家相关模块

// region:    --- 资源路径与常量
const PLAYER_SPRITE: &str = "player_a_01.png"; // 玩家精灵图路径
const PLAYER_SIZE: (f32, f32) = (144., 75.); // 玩家精灵尺寸
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png"; // 玩家激光精灵图路径
const PLAYER_LASER_SIZE: (f32, f32) = (9., 54.); // 玩家激光尺寸

const ENEMY_SPRITE: &str = "enemy_a_01.png"; // 敌人精灵图路径
const ENEMY_SIZE: (f32, f32) = (144., 75.); // 敌人精灵尺寸
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png"; // 敌人激光精灵图路径
const ENEMY_LASER_SIZE: (f32, f32) = (17., 55.); // 敌人激光尺寸

const EXPLOSION_SHEET: &str = "explo_a_sheet.png"; // 爆炸精灵图集路径
const EXPLOSION_LEN: usize = 16; // 爆炸动画帧数
const ENEMY_EXPLOSION_SOUND: &str = "enemy_explosion.ogg"; // 敌人爆炸音效路径

const SPRITE_SCALE: f32 = 0.5; // 精灵缩放比例
// endregion: --- 资源路径与常量

// region:    --- 游戏核心常量
const BASE_SPEED: f32 = 500.; // 基础移动速度

const PLAYER_RESPAWN_DELAY: f64 = 2.; // 玩家重生延迟（秒）
const ENEMY_MAX: u32 = 2; // 最大敌人数量
const FORMATION_MEMBERS_MAX: u32 = 2; // 编队最大成员数
// endregion: --- 游戏核心常量

// region:    --- 资源结构体定义
#[derive(Resource)]
pub struct WinSize {
    pub w: f32, // 窗口宽度
    pub h: f32, // 窗口高度
}

#[derive(Resource)]
struct GameTextures {
    player: Handle<Image>,                        // 玩家精灵资源句柄
    player_laser: Handle<Image>,                  // 玩家激光精灵资源句柄
    enemy: Handle<Image>,                         // 敌人精灵资源句柄
    enemy_laser: Handle<Image>,                   // 敌人激光精灵资源句柄
    explosion_layout: Handle<TextureAtlasLayout>, // 爆炸精灵图集布局句柄
    explosion_texture: Handle<Image>,             // 爆炸精灵图资源句柄
    enemy_explosion_sound: Handle<AudioSource>,   // 敌人爆炸音效资源句柄
}

#[derive(Resource)]
struct EnemyCount(u32); // 当前敌人数量（资源形式存储）

#[derive(Resource)]
struct PlayerState {
    on: bool,       // 玩家是否存活
    last_shot: f64, // 最后一次死亡时间（-1表示未死亡过）
}

// PlayerState默认实现
impl Default for PlayerState {
    fn default() -> Self {
        Self {
            on: false,      // 初始状态：玩家未存活
            last_shot: -1., // 初始无死亡记录
        }
    }
}

// PlayerState方法扩展
impl PlayerState {
    // 标记玩家死亡，记录死亡时间
    pub fn shot(&mut self, time: f64) {
        self.on = false;
        self.last_shot = time;
    }

    // 标记玩家重生，重置死亡时间
    pub fn spawned(&mut self) {
        self.on = true;
        self.last_shot = -1.;
    }
}

// 自定义事件：敌人爆炸事件（用于触发音效等逻辑）
#[derive(Event)]
struct EnemyExplosionEvent;
// endregion: --- 资源结构体定义

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.04))) // 设置背景颜色
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            // 添加默认插件并配置窗口
            primary_window: Some(Window {
                title: "Rust Invaders!".into(),  // 窗口标题
                resolution: (598., 676.).into(), // 窗口分辨率
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(PlayerPlugin) // 添加玩家系统插件
        .add_plugins(EnemyPlugin) // 添加敌人系统插件
        .add_event::<EnemyExplosionEvent>() // 注册敌人爆炸事件
        .add_systems(Startup, setup_system) // 启动阶段执行：初始化系统
        .add_systems(Update, movable_system) // 每帧执行：可移动实体逻辑
        .add_systems(Update, player_laser_hit_enemy_system) // 每帧执行：玩家激光命中敌人逻辑
        .add_systems(Update, enemy_laser_hit_player_system) // 每帧执行：敌人激光命中玩家逻辑
        .add_systems(Update, explosion_to_spawn_system) // 每帧执行：爆炸生成逻辑
        .add_systems(Update, explosion_animation_system) // 每帧执行：爆炸动画逻辑
        .add_systems(Update, enemy_explosion_audio_system) // 每帧执行：敌人爆炸音效逻辑
        .run();
}

// 初始化系统：加载资源、设置窗口尺寸、创建摄像机等
fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<&Window, With<PrimaryWindow>>,
) {
    // 生成2D摄像机
    commands.spawn(Camera2d);

    // 获取窗口尺寸
    let Ok(primary) = query.get_single() else {
        return;
    };
    let (win_w, win_h) = (primary.width(), primary.height());

    // 存储窗口尺寸资源
    let win_size = WinSize { w: win_w, h: win_h };
    commands.insert_resource(win_size);

    // 创建爆炸精灵图集
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlasLayout::from_grid(UVec2::new(64, 64), 4, 4, None, None);
    let explosion_layout = texture_atlases.add(texture_atlas);

    // 加载敌人爆炸音效
    let enemy_explosion_sound = asset_server.load(ENEMY_EXPLOSION_SOUND);

    // 存储游戏纹理资源
    let game_textures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        player_laser: asset_server.load(PLAYER_LASER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        enemy_laser: asset_server.load(ENEMY_LASER_SPRITE),
        explosion_layout,
        explosion_texture: texture_handle,
        enemy_explosion_sound,
    };
    commands.insert_resource(game_textures);
    commands.insert_resource(EnemyCount(0)); // 初始化敌人数量为0
}

// 可移动实体逻辑：处理实体移动、超出屏幕自动销毁
fn movable_system(
    mut commands: Commands,
    time: Res<Time>,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>,
) {
    let delta = time.delta_secs(); // 帧时间间隔

    for (entity, velocity, mut transform, movable) in &mut query {
        let translation = &mut transform.translation;
        // 根据速度和时间更新位置
        translation.x += velocity.x * delta * BASE_SPEED;
        translation.y += velocity.y * delta * BASE_SPEED;

        // 自动销毁逻辑：超出屏幕范围时销毁
        if movable.auto_despawn {
            const MARGIN: f32 = 200.; // 超出屏幕的边距
            let out_of_bounds = translation.y > win_size.h / 2. + MARGIN
                || translation.y < -win_size.h / 2. - MARGIN
                || translation.x > win_size.w / 2. + MARGIN
                || translation.x < -win_size.w / 2. - MARGIN;

            if out_of_bounds {
                commands.entity(entity).despawn();
            }
        }
    }
}

// 玩家激光命中敌人逻辑：处理碰撞检测、敌人销毁、爆炸生成
#[allow(clippy::type_complexity)] // 允许复杂的查询类型
fn player_laser_hit_enemy_system(
    mut commands: Commands,
    mut enemy_count: ResMut<EnemyCount>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromPlayer>)>,
    enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>,
    mut enemy_explosion_events: EventWriter<EnemyExplosionEvent>,
) {
    let mut despawned_entities = HashSet::new(); // 记录已销毁的实体

    // 遍历所有玩家激光
    for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
        if despawned_entities.contains(&laser_entity) {
            continue; // 跳过已销毁的激光
        }

        let laser_scale = laser_tf.scale.xy(); // 获取激光缩放比例

        // 遍历所有敌人
        for (enemy_entity, enemy_tf, enemy_size) in enemy_query.iter() {
            if despawned_entities.contains(&enemy_entity)
                || despawned_entities.contains(&laser_entity)
            {
                continue; // 跳过已销毁的敌人或激光
            }

            let enemy_scale = enemy_tf.scale.xy(); // 获取敌人缩放比例

            // 碰撞检测：用轴对齐包围盒（AABB）判断
            let laser_aabb = Aabb2d::new(
                laser_tf.translation.truncate(),
                (laser_size.0 * laser_scale) / 2.,
            );
            let enemy_aabb = Aabb2d::new(
                enemy_tf.translation.truncate(),
                (enemy_size.0 * enemy_scale) / 2.,
            );

            if laser_aabb.intersects(&enemy_aabb) {
                // 销毁敌人
                commands.entity(enemy_entity).despawn();
                despawned_entities.insert(enemy_entity);
                enemy_count.0 -= 1; // 减少敌人数量

                // 销毁激光
                commands.entity(laser_entity).despawn();
                despawned_entities.insert(laser_entity);

                // 生成爆炸
                commands.spawn(ExplosionToSpawn(enemy_tf.translation));

                // 发送敌人爆炸事件（用于触发音效）
                enemy_explosion_events.send(EnemyExplosionEvent);
            }
        }
    }
}

// 敌人激光命中玩家逻辑：处理碰撞检测、玩家销毁、爆炸生成
#[allow(clippy::type_complexity)] // 允许复杂的查询类型
fn enemy_laser_hit_player_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromEnemy>)>,
    player_query: Query<(Entity, &Transform, &SpriteSize), With<Player>>,
) {
    // 获取玩家实体（游戏中应该只有一个玩家）
    if let Ok((player_entity, player_tf, player_size)) = player_query.get_single() {
        let player_scale = player_tf.scale.xy(); // 获取玩家缩放比例

        // 遍历所有敌人激光
        for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
            let laser_scale = laser_tf.scale.xy(); // 获取激光缩放比例

            // 碰撞检测：用轴对齐包围盒（AABB）判断
            let laser_aabb = Aabb2d::new(
                laser_tf.translation.truncate(),
                (laser_size.0 * laser_scale) / 2.,
            );
            let player_aabb = Aabb2d::new(
                player_tf.translation.truncate(),
                (player_size.0 * player_scale) / 2.,
            );

            if laser_aabb.intersects(&player_aabb) {
                // 销毁玩家
                commands.entity(player_entity).despawn();
                player_state.shot(time.elapsed_secs_f64()); // 记录死亡时间

                // 销毁激光
                commands.entity(laser_entity).despawn();

                // 生成爆炸
                commands.spawn(ExplosionToSpawn(player_tf.translation));

                break; // 玩家死亡后跳出循环
            }
        }
    }
}

// 爆炸生成逻辑：将ExplosionToSpawn转换为实际爆炸精灵
fn explosion_to_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    query: Query<(Entity, &ExplosionToSpawn)>,
) {
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
        // 生成爆炸精灵
        commands
            .spawn((
                Sprite {
                    image: game_textures.explosion_texture.clone(), // 爆炸精灵图
                    texture_atlas: Some(TextureAtlas {
                        // 精灵图集配置
                        layout: game_textures.explosion_layout.clone(),
                        index: 0, // 从第一帧开始播放
                    }),
                    ..Default::default()
                },
                Transform::from_translation(explosion_to_spawn.0), // 爆炸位置
            ))
            .insert(Explosion) // 标记为爆炸实体
            .insert(ExplosionTimer::default()); // 爆炸动画计时器

        // 销毁ExplosionToSpawn标记实体
        commands.entity(explosion_spawn_entity).despawn();
    }
}

// 爆炸动画逻辑：处理爆炸帧更新、动画结束销毁
fn explosion_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionTimer, &mut Sprite), With<Explosion>>,
) {
    for (entity, mut timer, mut sprite) in &mut query {
        timer.0.tick(time.delta()); // 推进动画计时器

        if timer.0.finished() {
            // 计时器触发（切换爆炸帧）
            if let Some(texture) = sprite.texture_atlas.as_mut() {
                texture.index += 1; // 切换到下一帧

                // 动画播放完毕：销毁爆炸实体
                if texture.index >= EXPLOSION_LEN {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

// 敌人爆炸音效逻辑：响应EnemyExplosionEvent播放音效
fn enemy_explosion_audio_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    mut events: EventReader<EnemyExplosionEvent>,
) {
    for _ in events.read() {
        // 播放敌人爆炸音效（单次播放）
        commands.spawn((
            AudioPlayer::new(game_textures.enemy_explosion_sound.clone()),
            PlaybackSettings::ONCE,
        ));
    }
}
