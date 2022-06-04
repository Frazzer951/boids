use bevy::prelude::Component;

// region - Common Components
#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}
impl Velocity {
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn normalize(&mut self) -> &mut Self {
        let mag = self.magnitude();
        self.x /= mag;
        self.y /= mag;
        self
    }
}

#[derive(Component)]
pub struct Movable;
// endregion

// region - Boid Components
#[derive(Component)]
pub struct Boid;
// endregion
