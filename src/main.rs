use std::f32::consts::PI;

use bevy::prelude::*;

use crate::components::{Boid, Movable, Velocity};

mod components;

// region - Game Constants
// endregion

// region - Resources
pub struct WinSize {
    pub w: f32,
    pub h: f32,
}
// endregion

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_system)
        .add_system(movable_system)
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

    // spawn single "boid"
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(20., 20.)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0., 0., 0.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Boid)
        .insert(Velocity {
            magnitude: 5.,
            angle: PI / 4.,
        })
        .insert(Movable);
}

fn movable_system(
    win_size: Res<WinSize>,
    mut query: Query<(&Velocity, &mut Transform), With<Movable>>,
) {
    for (velocity, mut transform) in query.iter_mut() {
        // Get X and Y components
        let x = velocity.magnitude * velocity.angle.cos();
        let y = velocity.magnitude * velocity.angle.sin();
        /*println!(
            "x: {}, y: {}, angle: {}, magnitude: {}",
            x, y, velocity.angle, velocity.magnitude
        );*/

        // Update position
        let translation = &mut transform.translation;
        translation.x += x;
        translation.y += y;

        translation.x = wrap(translation.x, -win_size.w / 2., win_size.w / 2.);
        translation.y = wrap(translation.y, -win_size.h / 2., win_size.h / 2.);

        transform.rotation = Quat::from_rotation_z(velocity.angle);
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
