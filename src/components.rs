use bevy::prelude::Component;

// region - Common Components
#[derive(Component)]
pub struct Velocity {
    pub magnitude: f32,
    pub angle: f32,
}

#[derive(Component)]
pub struct Movable;
// endregion

// region - Boid Components
#[derive(Component)]
pub struct Boid;
// endregion
