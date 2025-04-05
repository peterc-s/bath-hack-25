use bevy::prelude::*;
use strum::{EnumDiscriminants, EnumIter};

#[derive(Component, Default)]
pub struct Bonnie {
    pub state: BonnieState,
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, EnumIter, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
pub enum BonnieState {
    #[default]
    Idle,
    Walking(IVec2),
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
