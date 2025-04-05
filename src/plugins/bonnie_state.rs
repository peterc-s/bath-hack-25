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

///////
// Plugin
///////

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

///////
// Pooping
///////

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

///////
// Movement
///////

macro_rules! move_towards_target {
    ($window:expr, $state_machine:expr, $target_pos:expr, $move_speed:expr) => {{
        let current_pos = match $window.position {
            WindowPosition::At(pos) => pos,
            _ => IVec2::new(100, 100),
        };

        let diff = $target_pos - current_pos;
        let len = diff.length_squared().isqrt();
        let mut move_size = $move_speed;

        if len < move_size {
            move_size = len;
        }

        if move_size == 0 {
            $state_machine.unblock();
            let remaining = $state_machine.timer.remaining();
            $state_machine.timer.tick(remaining);
        } else {
            let move_vec = if len > 0 {
                let x_norm = (diff.x as f64 / len as f64) * move_size as f64;
                let y_norm = (diff.y as f64 / len as f64) * move_size as f64;
                IVec2::new(x_norm.round() as i32, y_norm.round() as i32)
            } else {
                diff
            };

            $window.position = WindowPosition::At(current_pos + move_vec);
        }
    }};

    ($window:expr, $target_pos:expr, $state_machine:expr) => {
        move_towards_target!($window, $state_machine, $target_pos, 5)
    };
}

///////
// State
///////

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
    cursor_pos: Res<GlobalCursorPosition>,
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
                move_towards_target!(window, machine, to, 5);
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
                // get cursor position
                if let Some(to) = cursor_pos.0 {
                    let to = to.as_ivec2() - IVec2::new(150, 150);
                    move_towards_target!(window, machine, to, 10);
                }
            }
        }
    }
}
