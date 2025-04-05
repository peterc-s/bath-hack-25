use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Bonnie {
    pub state: BonnieState,
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BonnieState {
    #[default]
    Idle,
    Walking,
}

#[derive(Component, Debug)]
pub struct StateMachine {
    pub timer: Timer,
    pub can_change: bool,
}

impl StateMachine {
    pub fn block(&mut self) {
        self.can_change = false;
    }

    pub fn unblock(&mut self) {
        self.can_change = true;
    }

    pub fn toggle_block(&mut self) {
        self.can_change = !self.can_change;
    }
}
