use std::f32::consts::PI;

use bevy::math::{vec2, vec3};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rand::{thread_rng, Rng};

use crate::components::{Boid, Movable, Velocity};

mod components;

// region - Game Settings
const BOID_COHERENCE: f32 = 2.;
const BOID_SEPARATION: f32 = 40.;
const SEPARATION_DISTANCE: f32 = 30.;
const BOID_ALIGNMENT: f32 = 60.;
// endregion

// region - Game Constants
const BOID_ROTATE_OFFSET: f32 = -PI / 2.;
const BOID_SCALE: f32 = 5.;
const BOID_BASE_SPEED: f32 = 200.;
const BOID_MIN_SPEED: f32 = 100.;
const BOID_MAX_SPEED: f32 = 500.;
const NUMBER_OF_BOIDS: usize = 200;
// endregion

// region - Resources
pub struct WinSize {
    pub w: f32,
    pub h: f32,
}
// endregion

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Boids".to_string(),
            width: 1000.,
            height: 800.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup_system)
        .add_startup_system_to_stage(StartupStage::PostStartup, boid_spawn_system)
        .add_system(movable_system)
        .add_system(update_boid_system)
        .run()
}

fn setup_system(mut commands: Commands, mut windows: ResMut<Windows>) {
    // camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // capture window size
    let window = windows.get_primary_mut().unwrap();
    let (win_w, win_h) = (window.width(), window.height());

    // add WinSize resource
    let win_size = WinSize { w: win_w, h: win_h };
    commands.insert_resource(win_size);
}

fn boid_spawn_system(mut commands: Commands, win_size: Res<WinSize>) {
    let shape = shapes::Polygon {
        points: vec![vec2(-1., -2.), vec2(1., -2.), vec2(0., 2.)],
        closed: true,
    };
    let mut rng = thread_rng();

    let mut spawn_boid = |x: f32, y: f32, x_comp: f32, y_comp| {
        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shape,
                DrawMode::Outlined {
                    fill_mode: FillMode::color(Color::CYAN),
                    outline_mode: StrokeMode::new(Color::BLACK, 0.1),
                },
                Transform {
                    translation: vec3(x, y, 0.),
                    rotation: Default::default(),
                    scale: vec3(BOID_SCALE, BOID_SCALE, 1.),
                },
            ))
            .insert(Boid)
            .insert(Velocity {
                x: x_comp,
                y: y_comp,
            })
            .insert(Movable);
    };

    // spawn boids
    for _ in 0..NUMBER_OF_BOIDS {
        let angle = (rng.gen_range(0..360) as f32) * (PI / 180.);

        spawn_boid(
            rng.gen_range(-win_size.w..win_size.w) as f32,
            rng.gen_range(-win_size.h..win_size.h) as f32,
            angle.cos() * BOID_BASE_SPEED,
            angle.sin() * BOID_BASE_SPEED,
        );
    }
}

fn movable_system(
    win_size: Res<WinSize>,
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform), With<Movable>>,
) {
    let time_delta = time.delta_seconds();
    for (velocity, mut transform) in query.iter_mut() {
        // Get X and Y components
        let x = velocity.x * time_delta;
        let y = velocity.y * time_delta;
        let angle = (if velocity.x == 0. {
            0_f32
        } else {
            velocity.y / velocity.x
        })
        .atan()
            + if velocity.x < 0. { PI } else { 0. };

        // Update position
        let translation = &mut transform.translation;
        translation.x += x;
        translation.y += y;

        translation.x = wrap(translation.x, -win_size.w / 2., win_size.w / 2.);
        translation.y = wrap(translation.y, -win_size.h / 2., win_size.h / 2.);

        transform.rotation = Quat::from_rotation_z(angle + BOID_ROTATE_OFFSET);
    }
}

fn update_boid_system(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Boid>>,
) {
    let time_delta = time.delta_seconds();
    let positions: Vec<(f32, f32)> = query
        .iter()
        .map(|(_vel, tf)| (tf.translation.x, tf.translation.y))
        .collect();
    let sum_positions: (f32, f32) = positions
        .iter()
        .fold((0., 0.), |acc, (x, y)| (acc.0 + x, acc.1 + y));
    let sum_velocities = query
        .iter()
        .fold((0., 0.), |acc, (vel, _tf)| (acc.0 + vel.x, acc.1 + vel.y));

    for (mut velocity, tf) in query.iter_mut() {
        let translation = tf.translation;
        // rule 1 -- coherence - center of mass
        let sum_positions = (
            (sum_positions.0 - translation.x) / (NUMBER_OF_BOIDS as f32 - 1.),
            (sum_positions.1 - translation.y) / (NUMBER_OF_BOIDS as f32 - 1.),
        );
        let r1_vec = (
            BOID_COHERENCE * (sum_positions.0 - translation.x) / 100.,
            BOID_COHERENCE * (sum_positions.1 - translation.y) / 100.,
        );

        // rule 2 -- separation - keep distance
        let mut r2_vec = (0., 0.);
        let cur_pos = &(translation.x, translation.y);
        for pos in &positions {
            if pos != cur_pos && distance(cur_pos, pos) < SEPARATION_DISTANCE {
                let x_diff = pos.0 - cur_pos.0;
                let y_diff = pos.1 - cur_pos.1;
                r2_vec.0 -= x_diff;
                r2_vec.1 -= y_diff;
            }
        }
        r2_vec = (
            BOID_SEPARATION * r2_vec.0 / 100.,
            BOID_SEPARATION * r2_vec.1 / 100.,
        );

        // rule 3 -- alignment - match velocities
        let sum_velocities = (
            (sum_velocities.0 - velocity.x) / (NUMBER_OF_BOIDS as f32 - 1.),
            (sum_velocities.1 - velocity.y) / (NUMBER_OF_BOIDS as f32 - 1.),
        );
        let r3_vec = (
            BOID_ALIGNMENT * (sum_velocities.0 - velocity.x) / 100.,
            BOID_ALIGNMENT * (sum_velocities.1 - velocity.y) / 100.,
        );

        // update velocity
        velocity.x += r1_vec.0 + r2_vec.0 + r3_vec.0 * time_delta;
        velocity.y += r1_vec.1 + r2_vec.1 + r3_vec.1 * time_delta;

        // normalize velocities
        let boid_speed = velocity.magnitude().max(BOID_MIN_SPEED).min(BOID_MAX_SPEED);
        velocity.normalize();
        velocity.x *= boid_speed;
        velocity.y *= boid_speed;
    }
}

fn wrap(val: f32, min: f32, max: f32) -> f32 {
    if val > max {
        min
    } else if val < min {
        max
    } else {
        val
    }
}

fn distance(p1: &(f32, f32), p2: &(f32, f32)) -> f32 {
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    (dx * dx + dy * dy).sqrt()
}
