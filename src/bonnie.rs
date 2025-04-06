use crate::plugins::bonnie_state::BonnieState;
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Bonnie {
    pub state: BonnieState,
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

    pub fn finish(&mut self) {
        self.can_change = true;
        let remaining = self.timer.remaining();
        self.timer.tick(remaining);
    }

    pub fn toggle_block(&mut self) {
        self.can_change = !self.can_change;
    }
}
