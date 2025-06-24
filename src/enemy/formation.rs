use crate::{BASE_SPEED, FORMATION_MEMBERS_MAX, WinSize};
use bevy::prelude::{Component, Resource};
use rand::{Rng, thread_rng};
use std::f32::consts::PI;

/// Component - Enemy Formation (per enemy)
#[derive(Clone, Component)]
pub struct Formation {
    pub start: (f32, f32),
    pub radius: (f32, f32),
    pub pivot: (f32, f32),
    pub speed: f32,
    pub angle: f32,               // change per tick
    pub change_timer: f32,        // timer for parameter changes
    pub pivot_delta: (f32, f32),  // pivot change speed
    pub radius_delta: (f32, f32), // radius change speed
    pub speed_delta: f32,         // speed change rate
}

/// Resource - Formation Maker
#[derive(Default, Resource)]
pub struct FormationMaker {
    current_template: Option<Formation>,
    current_members: u32,
}

/// Formation factory implementation
impl FormationMaker {
    pub fn make(&mut self, win_size: &WinSize) -> Formation {
        match (
            &self.current_template,
            self.current_members >= FORMATION_MEMBERS_MAX,
        ) {
            // if has current template and still within max members
            (Some(tmpl), false) => {
                self.current_members += 1;
                tmpl.clone()
            }
            // if first formation or previous formation is full (need to create a new one)
            (None, _) | (_, true) => {
                let mut rng = thread_rng();

                // compute the start x/y
                let w_span = win_size.w / 2. + 100.;
                let h_span = win_size.h / 2. + 100.;
                let x = if rng.gen_bool(0.5) { w_span } else { -w_span };
                let y = rng.gen_range(-h_span..h_span);
                let start = (x, y);

                // compute the pivot x/y
                let w_span = win_size.w / 4.;
                let h_span = win_size.h / 3. - 50.;
                let pivot = (rng.gen_range(-w_span..w_span), rng.gen_range(0.0..h_span));

                // compute the radius
                let radius = (rng.gen_range(80.0..150.), 100.);

                // compute the start angle
                let angle = (y - pivot.1).atan2(x - pivot.0);

                // speed (fixed for now)
                let speed = BASE_SPEED;

                // 随机生成参数变化速度
                let pivot_delta = (rng.gen_range(-20.0..20.0), rng.gen_range(-20.0..20.0));
                let radius_delta = (rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0));
                let speed_delta = rng.gen_range(-10.0..10.0);

                // create the formation
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

                // store as template
                self.current_template = Some(formation.clone());
                // reset members to 1
                self.current_members = 1;

                formation
            }
        }
    }
}
