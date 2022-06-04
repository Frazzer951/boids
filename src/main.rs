use std::f32::consts::PI;

use bevy::math::{vec2, vec3};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rand::{Rng, thread_rng};

use crate::components::{Boid, Movable, Velocity};

mod components;

// region - Game Constants
const BOID_ROTATE_OFFSET: f32 = -PI / 2.;
const BOID_SCALE: f32 = 10.;
const BOID_SPEED: f32 = 200.;
const NUMBER_OF_BOIDS: usize = 10;
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
            angle.cos() * BOID_SPEED,
            angle.sin() * BOID_SPEED,
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
            0.
        } else {
            velocity.y / velocity.x
        })
        .atan() + if velocity.x < 0. {PI} else {0.};


        // Update position
        let translation = &mut transform.translation;
        translation.x += x;
        translation.y += y;

        translation.x = wrap(translation.x, -win_size.w / 2., win_size.w / 2.);
        translation.y = wrap(translation.y, -win_size.h / 2., win_size.h / 2.);

        println!(
            "x:{}, y:{}, angle:{}",
            velocity.x,
            velocity.y,
            angle * (180. / PI)
        );

        transform.rotation = Quat::from_rotation_z(angle + BOID_ROTATE_OFFSET);
    }
}

fn update_boid_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Boid>>,
) {
    let time_delta = time.delta_seconds();
    let rotation_speed = PI / 2.;

    let rotation_delta = if keyboard_input.pressed(KeyCode::A) {
        -rotation_speed * time_delta
    } else if keyboard_input.pressed(KeyCode::D) {
        rotation_speed * time_delta
    } else {
        0.
    };

    for mut velocity in query.iter_mut() {
        let rotation = rotate(rotation_delta, velocity.x, velocity.y);
        velocity.x = rotation.0;
        velocity.y = rotation.1;
    }
}

fn rotate(angle: f32, x: f32, y: f32) -> (f32, f32) {
    let s = angle.sin();
    let c = angle.cos();

    let x_prime = x * c + y * s;
    let y_prime = -x * s + y * c;
    (x_prime, y_prime)
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
