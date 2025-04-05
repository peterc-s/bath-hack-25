//! All the state stuff for Bonnie

use crate::{
    Bonnie, BonnieState, StateMachine, bonnie::BonnieStateDiscriminants, get_composite_mode,
    global_cursor::GlobalCursorPosition,
};
use bevy::{
    input::{ButtonState, mouse::MouseButtonInput},
    prelude::*,
    render::{camera::RenderTarget, view::RenderLayers},
    utils::Duration,
    window::{PrimaryWindow, WindowLevel, WindowRef},
    winit::WinitWindows,
};
use dpi::PhysicalSize;
use rand::{Rng, prelude::IteratorRandom};
use strum::IntoEnumIterator;

pub struct BonnieStatePlugin;

impl Plugin for BonnieStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalCursorPosition>().add_systems(
            Update,
            (
                state_transition,
                state_behaviours,
                close_poop_window_on_click,
            ),
        );
    }
}

fn random_state(current_state: BonnieState, screen_res: PhysicalSize<u32>) -> BonnieState {
    let mut rng = rand::rng();

    // randomly choose enum discriminant
    // that isn't the current one.
    let disc = BonnieStateDiscriminants::iter()
        .filter(|d| d != &current_state.into())
        .choose(&mut rng)
        .unwrap_or(BonnieState::Idle.into());

    // return an actual enum variant
    match disc {
        BonnieStateDiscriminants::Idle => BonnieState::Idle,
        BonnieStateDiscriminants::Walking => {
            // randomly generate a coordinate to go to with some buffer
            let x_min = 150;
            let x_max = screen_res.width.saturating_sub(150);
            let x_to = if x_max > x_min {
                rng.random_range(x_min..x_max)
            } else {
                rng.random_range(0..screen_res.width)
            };

            let y_min = 150;
            let y_max = screen_res.height.saturating_sub(150);
            let y_to = if y_max > y_min {
                rng.random_range(y_min..y_max)
            } else {
                rng.random_range(0..screen_res.height)
            };

            BonnieState::Walking((x_to as i32, y_to as i32).into())
        }
        BonnieStateDiscriminants::Pooping => BonnieState::Pooping,
        BonnieStateDiscriminants::Chasing => BonnieState::Chasing,
    }
}

fn state_transition(
    time: Res<Time>,
    mut query: Query<(&mut Bonnie, &mut StateMachine)>,
    winit_windows: NonSend<WinitWindows>,
    window_query: Query<Entity, With<PrimaryWindow>>,
) {
    let mut rng = rand::rng();

    // get bonnie and the state machine from the query
    for (mut bonnie, mut machine) in &mut query {
        // tick the timer
        machine.timer.tick(time.delta());

        // skip if the machine is blocked
        if !machine.can_change {
            continue;
        }

        // if the timer just finished
        if machine.timer.finished() {
            // get the monitor
            if let Some(monitor) = window_query
                .get_single()
                .ok()
                .and_then(|entity| winit_windows.get_window(entity))
                .and_then(|winit_window| winit_window.current_monitor())
            {
                // select a random bonnie state to switch to
                let new_state = random_state(bonnie.state, monitor.size());

                info!("Changing state from {:?} to {:?}.", bonnie.state, new_state);

                // switch to the selected state
                bonnie.state = new_state;

                // block on walking, unblocks and makes
                // timer finish when at correct coordinate
                if let BonnieState::Walking(_) = bonnie.state {
                    machine.block()
                }

                // reset timer
                machine
                    .timer
                    .set_duration(Duration::from_secs_f32(rng.random_range(1.0..4.0)));
                machine.timer.reset();
            }
        }
    }
}

fn state_behaviours(
    mut bonnie_query: Query<&mut Bonnie, With<Sprite>>,
    mut machine_query: Query<&mut StateMachine>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // get the state machine
    let mut machine = machine_query
        .get_single_mut()
        .expect("No state machine found.");

    // get the window
    let mut window = window_query
        .get_single_mut()
        .expect("No primary window found.");

    // get bonnie
    for bonnie in &mut bonnie_query {
        // do stuff based on the current bonnie state
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

                // unblock and make change state if at the point to walk to
                if move_size == 0 {
                    machine.unblock();

                    // make timer finish to change state
                    let remaining = machine.timer.remaining();
                    machine.timer.tick(remaining);

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
            BonnieState::Pooping => {
                // create the window with a poop window marker
                let poop_window = commands
                    .spawn((
                        Window {
                            transparent: true,
                            composite_alpha_mode: get_composite_mode(),
                            decorations: false,
                            resizable: false,
                            has_shadow: false,
                            titlebar_shown: false,
                            titlebar_transparent: false,
                            titlebar_show_buttons: false,
                            titlebar_show_title: false,
                            title: "Poop!".to_string(),
                            name: Some("bonnie.buddy".into()),
                            resolution: (10.0, 10.0).into(),
                            window_level: WindowLevel::AlwaysOnTop,
                            position: window.position,
                            ..default()
                        },
                        PoopWindowMarker,
                    ))
                    .id();

                // spawn a camera2d on render layer 1
                commands.spawn((
                    #[allow(deprecated)]
                    Camera2dBundle {
                        camera: Camera {
                            target: RenderTarget::Window(WindowRef::Entity(poop_window)),
                            ..default()
                        },
                        ..default()
                    },
                    RenderLayers::layer(42),
                ));

                // get the sprite
                let mut poop_sprite = Sprite::from_image(asset_server.load("BonPoop.png"));
                poop_sprite.custom_size = Some(Vec2::new(40.0, 40.0));

                // spawn the sprite on the render layer 1
                commands.spawn((poop_sprite, RenderLayers::layer(42)));

                // make timer finish to change state
                let remaining = machine.timer.remaining();
                machine.timer.tick(remaining);
            }
            BonnieState::Chasing => {
                todo!()
            }
        }
    }
}

#[derive(Component)]
struct PoopWindowMarker;

fn close_poop_window_on_click(
    mut commands: Commands,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    poop_windows: Query<(), With<PoopWindowMarker>>,
) {
    // get mouse events
    for event in mouse_button_events.read() {
        // if left click and is on a poop window
        if event.button == MouseButton::Left
            && event.state == ButtonState::Pressed
            && poop_windows.get(event.window).is_ok()
        {
            // despawn the poop window
            commands.entity(event.window).despawn_recursive();
        }
    }
}
