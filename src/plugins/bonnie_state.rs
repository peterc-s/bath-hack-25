use crate::{Bonnie, BonnieState, StateMachine, bonnie::BonnieStateDiscriminants};
use bevy::{prelude::*, utils::Duration, window::PrimaryWindow, winit::WinitWindows};
use dpi::PhysicalSize;
use rand::{Rng, prelude::IteratorRandom};
use strum::IntoEnumIterator;

pub struct BonnieStatePlugin;

impl Plugin for BonnieStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (state_transition, state_behaviours));
    }
}

fn random_state(current_state: BonnieState, screen_res: PhysicalSize<u32>) -> BonnieState {
    let mut rng = rand::rng();

    let disc = BonnieStateDiscriminants::iter()
        .filter(|d| d != &current_state.into())
        .choose(&mut rng)
        .unwrap_or(BonnieState::Idle.into());

    match disc {
        BonnieStateDiscriminants::Idle => BonnieState::Idle,
        BonnieStateDiscriminants::Walking => {
            let x_to = rng.random_range(0..screen_res.width);
            let y_to = rng.random_range(0..screen_res.height);

            BonnieState::Walking((x_to as i32, y_to as i32).into())
        }
    }
}

fn state_transition(
    time: Res<Time>,
    mut query: Query<(&mut Bonnie, &mut StateMachine)>,
    winit_windows: NonSend<WinitWindows>,
    window_query: Query<Entity, With<PrimaryWindow>>,
) {
    let mut rng = rand::rng();

    for (mut bonnie, mut machine) in &mut query {
        if !machine.can_change {
            continue;
        }

        machine.timer.tick(time.delta());

        if machine.timer.just_finished() {
            if let Some(monitor) = window_query
                .get_single()
                .ok()
                .and_then(|entity| winit_windows.get_window(entity))
                .and_then(|winit_window| winit_window.current_monitor())
            {
                let new_state = random_state(bonnie.state, monitor.size());

                info!("Changing state from {:?} to {:?}.", bonnie.state, new_state);

                bonnie.state = new_state;

                // block on walking, unblocks when at position
                if let BonnieState::Walking(_) = bonnie.state {
                    machine.block()
                }

                machine
                    .timer
                    .set_duration(Duration::from_secs_f32(rng.random_range(2.0..5.0)));
                machine.timer.reset();
            }
        }
    }
}

fn state_behaviours(
    mut bonnie_query: Query<&mut Bonnie, With<Sprite>>,
    mut machine_query: Query<&mut StateMachine>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut machine = machine_query
        .get_single_mut()
        .expect("No state machine found.");
    let mut window = window_query
        .get_single_mut()
        .expect("No primary window found.");

    for bonnie in &mut bonnie_query {
        match bonnie.state {
            BonnieState::Idle => {
                // do idle stuff
            }
            BonnieState::Walking(to) => {
                // get current window position
                let current_pos = match window.position {
                    WindowPosition::At(pos) => pos,
                    _ => IVec2::new(100, 100),
                };

                // default move size
                let mut move_size = 5;

                // get vector diff
                let diff = to - current_pos;
                // length of vector
                let len = diff.length_squared().isqrt();

                // if the move size is less than the length
                // use the length instead
                if len < move_size {
                    move_size = len;
                }

                // unblock if at the point to walk to
                if move_size == 0 {
                    machine.unblock();
                    continue;
                }

                // calculate the move vector
                let move_vec = if len >= 0 {
                    // get vector components as floats
                    let x_float = diff[0] as f64;
                    let y_float = diff[1] as f64;

                    // normalise and multiply by move size
                    let x_norm = (x_float / len as f64) * move_size as f64;
                    let y_norm = (y_float / len as f64) * move_size as f64;

                    IVec2::new(x_norm as i32, y_norm as i32)
                } else {
                    diff
                };

                // set new window position
                window.position = WindowPosition::At(current_pos + move_vec);
            }
        }
    }
}
