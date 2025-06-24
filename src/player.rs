use crate::components::{FromPlayer, Laser, Movable, Player, SpriteSize, Velocity};
use crate::{
    GameTextures, PLAYER_LASER_SIZE, PLAYER_RESPAWN_DELAY, PLAYER_SIZE, PlayerState, SPRITE_SCALE,
    WinSize,
};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerState::default())
            .add_systems(
                Update,
                player_spawn_system.run_if(on_timer(Duration::from_secs_f32(0.5))),
            )
            .add_systems(Update, player_keyboard_event_system)
            .add_systems(Update, player_fire_system);
    }
}

fn player_spawn_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    game_textures: Res<GameTextures>,
    win_size: Res<WinSize>,
) {
    let now = time.elapsed_secs_f64();
    let last_shot = player_state.last_shot;

    if !player_state.on && (last_shot == -1. || now > last_shot + PLAYER_RESPAWN_DELAY) {
        // add player
        let bottom = -win_size.h / 2.;
        commands
            .spawn((
                Sprite::from_image(game_textures.player.clone()),
                Transform {
                    translation: Vec3::new(
                        0.,
                        bottom + PLAYER_SIZE.1 / 2. * SPRITE_SCALE + 5.,
                        10.,
                    ),
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                    ..Default::default()
                },
            ))
            .insert(Player)
            .insert(SpriteSize::from(PLAYER_SIZE))
            .insert(Movable {
                auto_despawn: false,
            })
            .insert(Velocity { x: 0., y: 0. });

        player_state.spawned();
    }
}

fn player_fire_system(
    mut commands: Commands,
    kb: Res<ButtonInput<KeyCode>>,
    game_textures: Res<GameTextures>,
    query: Query<&Transform, With<Player>>,
) {
    if let Ok(player_tf) = query.get_single() {
        if kb.just_pressed(KeyCode::Space) {
            let (x, y) = (player_tf.translation.x, player_tf.translation.y);
            let x_offset = PLAYER_SIZE.0 / 2. * SPRITE_SCALE - 5.;

            let mut spawn_laser = |x_offset: f32| {
                commands
                    .spawn((
                        Sprite::from_image(game_textures.player_laser.clone()),
                        Transform {
                            translation: Vec3::new(x + x_offset, y + 15., 0.),
                            scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                            ..Default::default()
                        },
                    ))
                    .insert(Laser)
                    .insert(FromPlayer)
                    .insert(SpriteSize::from(PLAYER_LASER_SIZE))
                    .insert(Movable { auto_despawn: true })
                    .insert(Velocity { x: 0., y: 1. });
            };

            spawn_laser(x_offset);
            spawn_laser(-x_offset);
        }
    }
}

fn player_keyboard_event_system(
    kb: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    if let Ok(mut velocity) = query.get_single_mut() {
        velocity.x = if kb.pressed(KeyCode::ArrowLeft) {
            -1.
        } else if kb.pressed(KeyCode::ArrowRight) {
            1.
        } else {
            0.
        }
    }
}
