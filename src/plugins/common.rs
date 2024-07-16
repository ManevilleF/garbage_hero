use bevy::prelude::*;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Health>();
    }
}
#[derive(Debug, Component, Reflect)]
pub struct Health {
    pub current: u16,
    pub max: u16,
}

impl Health {
    pub const fn new(max: u16) -> Self {
        Self { current: max, max }
    }

    /// Returns `true` if health is still over 0
    pub fn damage(&mut self, amount: u16) -> bool {
        self.current = self.current.saturating_sub(amount);
        self.current > 0
    }

    pub fn heal(&mut self, amount: u16) {
        self.current += amount;
        self.current = self.current.min(self.max);
    }

    pub fn increase_max(&mut self, amount: u16) {
        self.max += amount;
        self.heal(amount);
    }
}
