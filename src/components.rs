use bevy::prelude::Component;

// region - Common Components
#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Movable;
// endregion

// region - Boid Components
#[derive(Component)]
pub struct Boid;
// endregion
