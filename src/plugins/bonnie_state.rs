use crate::{Bonnie, BonnieState, StateMachine};
use bevy::{prelude::*, utils::Duration, window::PrimaryWindow};
use rand::Rng;

pub struct BonnieStatePlugin;

impl Plugin for BonnieStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (state_transition, state_behaviours));
    }
}

fn state_transition(time: Res<Time>, mut query: Query<(&mut Bonnie, &mut StateMachine)>) {
    let mut rng = rand::rng();

    for (mut bonnie, mut machine) in &mut query {
        if !machine.can_change {
            continue;
        }

        machine.timer.tick(time.delta());

        if machine.timer.just_finished() {
            let new_state = match rng.random_range(0..2) {
                0 => BonnieState::Idle,
                1 => BonnieState::Walking,
                _ => unreachable!(),
            };

            info!("Changing state from {:?} to {:?}.", bonnie.state, new_state);

            bonnie.state = new_state;
            machine
                .timer
                .set_duration(Duration::from_secs_f32(rng.random_range(2.0..5.0)));
            machine.timer.reset();
        }
    }
}

fn state_behaviours(
    mut query: Query<&mut Bonnie>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut rng = rand::rng();
    if let Ok(mut window) = window_query.get_single_mut() {
        for bonnie in &mut query {
            match bonnie.state {
                BonnieState::Idle => {
                    // do idle stuff
                }
                BonnieState::Walking => {
                    // do walk animation
                    // move window in correct direction
                    let move_speed = 10;

                    // get current window position
                    let current_pos = match window.position {
                        WindowPosition::At(pos) => pos,
                        _ => IVec2::new(100, 100),
                    };

                    // get new position
                    let mut new_pos = current_pos;
                    match rng.random_range(0..4) {
                        0 => {
                            new_pos.x -= move_speed;
                        }
                        1 => {
                            new_pos.x += move_speed;
                        }
                        2 => {
                            new_pos.y -= move_speed;
                        }
                        3 => {
                            new_pos.y += move_speed;
                        }
                        _ => unreachable!(),
                    }

                    window.position = WindowPosition::At(new_pos);
                }
            }
        }
    }
}
