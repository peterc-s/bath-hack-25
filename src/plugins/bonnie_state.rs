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
    window::{PresentMode, PrimaryWindow, WindowLevel, WindowRef},
    winit::WinitWindows,
};
use dpi::PhysicalSize;
use rand::{
    Rng,
    prelude::{IndexedRandom, IteratorRandom},
};
use strum::IntoEnumIterator;

///////
// Plugin
///////

pub struct BonnieStatePlugin;

impl Plugin for BonnieStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalCursorPosition>()
            .add_systems(
                Update,
                (
                    state_transition,
                    state_behaviours,
                    close_poop_window_on_click,
                    close_teach_window_on_click,
                ),
            )
            .add_systems(Startup, add_poop_layer);
    }
}

///////
// Pooping
///////

#[derive(Component)]
struct PoopWindowMarker;

fn add_poop_layer(mut commands: Commands, asset_server: Res<AssetServer>) {
    // get the sprite
    let mut poop_sprite = Sprite::from_image(asset_server.load("BonPoop.png"));
    poop_sprite.custom_size = Some(Vec2::new(40.0, 40.0));

    // spawn the sprite on the render layer 1
    commands.spawn((poop_sprite, RenderLayers::layer(42)));
}

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

fn get_resolution_based_speed(screen_size: PhysicalSize<u32>, base_speed: f32) -> f32 {
    // calculate diagonal in pixels
    let diagonal = ((screen_size.width.pow(2) + screen_size.height.pow(2)) as f32).sqrt();

    // convert base speed (percentage of screen diagonal per second)
    let speed_ratio = 0.15;

    // minimum speed for very small screens
    diagonal * speed_ratio * base_speed
}

macro_rules! move_bonnie_to {
    ($window:expr, $state_machine:expr, $target_pos:expr, $move_speed:expr, $dt:expr) => {{
        let current_pos = match $window.position {
            WindowPosition::At(pos) => pos,
            _ => IVec2::new(100, 100),
        };

        let diff = $target_pos - current_pos;
        let len = diff.as_vec2().length();
        let move_per_frame = ($move_speed as f64 * $dt) as f32;
        let move_size = move_per_frame.min(len);

        if len <= move_per_frame {
            $state_machine.unblock();
            let remaining = $state_machine.timer.remaining();
            $state_machine.timer.tick(remaining);
        } else {
            let direction = diff.as_vec2().normalize();
            let move_vec = (direction * move_size).round().as_ivec2();

            $window.position = WindowPosition::At(current_pos + move_vec);
        }
    }};
}

///////
// Teaching
///////

#[derive(Component)]
struct TeachingWindowMarker;

fn close_teach_window_on_click(
    mut commands: Commands,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    teach_windows: Query<(), With<TeachingWindowMarker>>,
    mut machine_query: Query<&mut StateMachine>,
    render_layer_query: Query<(Entity, &RenderLayers)>,
) {
    // get mouse events
    for event in mouse_button_events.read() {
        // if left click and is on a teach window
        if event.button == MouseButton::Left
            && event.state == ButtonState::Pressed
            && teach_windows.get(event.window).is_ok()
        {
            // despawn the teach window
            commands.entity(event.window).despawn_recursive();

            // remove the things in the render layer
            for (entity, render_layers) in &render_layer_query {
                if *render_layers == RenderLayers::layer(43) {
                    commands.entity(entity).despawn_recursive();
                }
            }

            let mut machine = machine_query
                .get_single_mut()
                .expect("Failed to get state machine.");
            let remaining = machine.timer.remaining();
            machine.timer.tick(remaining);
            machine.unblock();
        }
    }
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
        BonnieStateDiscriminants::Teaching => BonnieState::Teaching,
    }
}

fn state_transition(
    time: Res<Time>,
    mut query: Query<(&mut Bonnie, &mut StateMachine)>,
    winit_windows: NonSend<WinitWindows>,
    window_query: Query<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
                match bonnie.state {
                    BonnieState::Walking(_) => machine.block(),
                    BonnieState::Chasing => machine.block(),
                    BonnieState::Teaching => {
                        machine.block();

                        let pos = WindowPosition::At(IVec2::new(-100, 300));

                        let teach_window = commands
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
                                    title: "Education!".to_string(),
                                    name: Some("bonnie.buddy".into()),
                                    resolution: (300.0, 300.0).into(),
                                    resize_constraints: WindowResizeConstraints {
                                        min_width: 300.0,
                                        min_height: 300.0,
                                        max_width: 300.0,
                                        max_height: 300.0,
                                    },
                                    present_mode: PresentMode::AutoNoVsync,
                                    window_level: WindowLevel::AlwaysOnTop,
                                    position: pos,
                                    ..default()
                                },
                                TeachingWindowMarker,
                            ))
                            .id();

                        // spawn a camera2d on render layer 1
                        commands.spawn((
                            #[allow(deprecated)]
                            Camera2dBundle {
                                camera: Camera {
                                    target: RenderTarget::Window(WindowRef::Entity(teach_window)),
                                    ..default()
                                },
                                ..default()
                            },
                            RenderLayers::layer(43),
                        ));

                        let education_sprites = [
                            "educational/meme1.png",
                            "educational/meme2.png",
                            "educational/meme3.png",
                        ];

                        // get the sprite
                        let mut teach_sprite = Sprite::from_image(
                            asset_server.load(
                                *education_sprites
                                    .choose(&mut rng)
                                    .expect("Couldn't get an education sprite."),
                            ),
                        );
                        teach_sprite.custom_size = Some(Vec2::new(300.0, 300.0));

                        // spawn the sprite on the render layer 1
                        commands.spawn((teach_sprite, RenderLayers::layer(43)));
                    }
                    _ => {}
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
    mut window_query: Query<&mut Window, (With<PrimaryWindow>, Without<TeachingWindowMarker>)>,
    window_entity_query: Query<Entity, With<PrimaryWindow>>,
    mut teach_window_query: Query<&mut Window, With<TeachingWindowMarker>>,
    mut commands: Commands,
    winit_windows: NonSend<WinitWindows>,
    cursor_pos: Res<GlobalCursorPosition>,
    time: Res<Time>,
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
        let monitor_size = window_entity_query
            .get_single()
            .ok()
            .and_then(|entity| winit_windows.get_window(entity))
            .and_then(|w| w.current_monitor())
            .map(|m| m.size())
            .unwrap_or(PhysicalSize::new(1920, 1080));
        // do stuff based on the current bonnie state
        match bonnie.state {
            BonnieState::Idle => {
                // do idle stuff
            }
            BonnieState::Walking(to) => {
                let speed = get_resolution_based_speed(monitor_size, 1.0);
                move_bonnie_to!(window, machine, to, speed, time.delta_secs_f64());
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
                            resolution: (40.0, 40.0).into(),
                            resize_constraints: WindowResizeConstraints {
                                min_width: 40.0,
                                min_height: 40.0,
                                max_width: 40.0,
                                max_height: 40.0,
                            },
                            present_mode: PresentMode::AutoNoVsync,
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

                // make timer finish to change state
                let remaining = machine.timer.remaining();
                machine.timer.tick(remaining);
            }
            BonnieState::Chasing => {
                let speed = get_resolution_based_speed(monitor_size, 2.0);
                // get cursor position
                if let Some(to) = cursor_pos.0 {
                    let to = to.as_ivec2() - IVec2::new(90, 147);
                    move_bonnie_to!(window, machine, to, speed, time.delta_secs_f64());
                }
            }
            BonnieState::Teaching => {
                if let Ok(mut teach_window) = teach_window_query.get_single_mut() {
                    let current_pos = match teach_window.position {
                        WindowPosition::At(pos) => pos,
                        _ => IVec2::new(100, 100),
                    };

                    let current_bonnie_pos = match window.position {
                        WindowPosition::At(pos) => pos,
                        _ => IVec2::new(100, 100),
                    };

                    let target_pos = current_bonnie_pos + IVec2::new(-150, 200);

                    let speed = get_resolution_based_speed(monitor_size, 0.8);
                    let diff = target_pos - current_pos;
                    let len = diff.as_vec2().length();
                    let move_per_frame = (speed as f64 * time.delta_secs_f64()) as f32;
                    let move_size = move_per_frame.min(len);

                    if len > move_per_frame {
                        let direction = diff.as_vec2().normalize();
                        let move_vec = (direction * move_size).round().as_ivec2();

                        teach_window.position = WindowPosition::At(current_pos + move_vec);
                    }
                }
            }
        }
    }
}
