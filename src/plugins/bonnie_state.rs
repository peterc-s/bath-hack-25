//! All the state stuff for Bonnie

use std::any::TypeId;

use crate::{
    bonnie::{Bonnie, StateMachine},
    get_composite_mode,
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
    Rng, SeedableRng, TryRngCore,
    prelude::{IndexedRandom, IteratorRandom},
    rngs::StdRng,
};
use strum::{EnumDiscriminants, EnumIter, IntoEnumIterator};

use super::global_cursor::GlobalCursorPosition;

////////
// Constants
////////

const WINDOW_SIZE_BUFFER: u32 = 150;
const POOP_LAYER: usize = 42;
const TEACH_LAYER: usize = 43;

////////
// Resources
////////

#[derive(Resource)]
struct GlobalRng(StdRng);

impl Default for GlobalRng {
    fn default() -> Self {
        let mut seed = [0; 32];
        rand::rngs::OsRng
            .try_fill_bytes(&mut seed)
            .expect("Couldn't seed GlobalRng.");
        Self(StdRng::from_seed(seed))
    }
}

////////
// States
////////

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, EnumIter, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
pub enum BonnieState {
    #[default]
    Idle,
    Walking(IVec2),
    Pooping,
    Chasing,
    Teaching,
}

impl From<BonnieStateDiscriminants> for BonnieState {
    fn from(value: BonnieStateDiscriminants) -> Self {
        match value {
            BonnieStateDiscriminants::Idle => BonnieState::Idle,
            BonnieStateDiscriminants::Walking => BonnieState::Walking(IVec2::ZERO),
            BonnieStateDiscriminants::Pooping => BonnieState::Pooping,
            BonnieStateDiscriminants::Chasing => BonnieState::Chasing,
            BonnieStateDiscriminants::Teaching => BonnieState::Teaching,
        }
    }
}

///////
// Plugin
///////

pub struct BonnieStatePlugin;

impl Plugin for BonnieStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<BonnieState>()
            .init_resource::<GlobalRng>()
            .add_systems(Startup, setup_poop_sprite)
            .add_systems(PostUpdate, handle_state_transitions)
            .add_systems(
                Update,
                (
                    handle_window_closing::<PoopWindow>,
                    handle_window_closing::<TeachWindow>,
                    handle_movement,
                    handle_teaching,
                    handle_chasing,
                )
                    .chain(),
            )
            .add_systems(OnEnter(BonnieState::Teaching), setup_teaching)
            .add_systems(OnEnter(BonnieState::Pooping), setup_pooping);
    }
}

///////
// State Management
///////

fn handle_state_transitions(
    time: Res<Time>,
    mut bonnie: Query<&mut Bonnie>,
    mut machine: Query<&mut StateMachine>,
    winit_windows: NonSend<WinitWindows>,
    window_query: Query<Entity, With<PrimaryWindow>>,
    mut next_state: ResMut<NextState<BonnieState>>,
    mut rng: ResMut<GlobalRng>,
) {
    // get machine and bonnie
    let mut machine = machine.single_mut();
    let mut bonnie = bonnie.single_mut();

    // tick the machine timer
    machine.timer.tick(time.delta());

    // if the machine can change state and is finished
    if machine.can_change && machine.timer.finished() {
        // get the monitor
        let monitor = window_query
            .get_single()
            .ok()
            .and_then(|entity| winit_windows.get_window(entity))
            .and_then(|winit_window| winit_window.current_monitor())
            .expect("Failed to get monitor.");

        // generate a new random state
        let new_state = random_state(&bonnie.state, &mut rng.0, monitor.size());
        info!("Changing state from {:?} to {:?}.", bonnie.state, new_state);

        // set the state
        next_state.set(new_state.clone());
        bonnie.state = new_state;

        // reset timer
        machine.timer.reset();
        machine
            .timer
            .set_duration(Duration::from_secs_f32(rng.0.random_range(1.0..4.0)));
        info!("Timer reset to: {:?}", machine.timer.remaining());
    }
}

fn random_state(
    current: &BonnieState,
    rng: &mut impl Rng,
    monitor_size: PhysicalSize<u32>,
) -> BonnieState {
    let mut next_state = BonnieStateDiscriminants::iter()
        .filter(|d| *d != BonnieStateDiscriminants::from(current))
        .choose(rng)
        .map_or(BonnieState::Idle, |disc| match disc {
            BonnieStateDiscriminants::Walking => {
                let x_range = WINDOW_SIZE_BUFFER..(monitor_size.width - WINDOW_SIZE_BUFFER);
                let y_range = WINDOW_SIZE_BUFFER..(monitor_size.height - WINDOW_SIZE_BUFFER);
                BonnieState::Walking(IVec2::new(
                    rng.random_range(x_range) as i32,
                    rng.random_range(y_range) as i32,
                ))
            }
            _ => BonnieState::from(disc),
        });

    next_state = match next_state {
        BonnieState::Walking(_) => {
            // randomly generate a coordinate to go to with some buffer
            let x_min = 150;
            let x_max = monitor_size.width.saturating_sub(150);
            let x_to = if x_max > x_min {
                rng.random_range(x_min..x_max)
            } else {
                rng.random_range(0..monitor_size.width)
            };

            let y_min = 150;
            let y_max = monitor_size.height.saturating_sub(150);
            let y_to = if y_max > y_min {
                rng.random_range(y_min..y_max)
            } else {
                rng.random_range(0..monitor_size.height)
            };

            BonnieState::Walking((x_to as i32, y_to as i32).into())
        }
        _ => next_state,
    };

    info!(
        "Current: {:?}, Next: {:?}",
        BonnieStateDiscriminants::from(current),
        next_state
    );

    next_state
}

///////
// Window management
///////

#[derive(Component)]
struct PoopWindow;

#[derive(Component)]
struct TeachWindow;

fn handle_window_closing<T: Component>(
    mut commands: Commands,
    mut mouse_events: EventReader<MouseButtonInput>,
    windows: Query<(), With<T>>,
    mut machine: Query<&mut StateMachine>,
    render_layer_query: Query<(Entity, &RenderLayers)>,
) {
    for event in mouse_events.read() {
        if event.button == MouseButton::Left
            && event.state == ButtonState::Pressed
            && windows.get(event.window).is_ok()
        {
            commands.entity(event.window).despawn_recursive();

            if TypeId::of::<T>() == TypeId::of::<TeachWindow>() {
                // finish state machine
                if let Ok(mut machine) = machine.get_single_mut() {
                    machine.finish();
                }

                // clear render layer ready for next image
                for (entity, render_layers) in &render_layer_query {
                    if *render_layers == RenderLayers::layer(TEACH_LAYER) {
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
        }
    }
}

///////
// Movement system
///////

fn handle_movement(
    time: Res<Time>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    winit_windows: NonSend<WinitWindows>,
    window_entity_query: Query<Entity, With<PrimaryWindow>>,
    state: Res<State<BonnieState>>,
    cursor_pos: Res<GlobalCursorPosition>,
) {
    let Ok(mut window) = window_query.get_single_mut() else {
        return;
    };

    let monitor = window_entity_query
        .get_single()
        .ok()
        .and_then(|entity| winit_windows.get_window(entity))
        .and_then(|winit_window| winit_window.current_monitor())
        .expect("Failed to get monitor.");

    let target_position = match *state.get() {
        BonnieState::Walking(target) => target,
        BonnieState::Chasing => cursor_pos
            .0
            .map(|v| v.as_ivec2() - IVec2::new(90, 147))
            .expect("Cursor position not available"),
        _ => return,
    };

    let current_position = match window.position {
        WindowPosition::At(pos) => pos,
        _ => IVec2::ZERO,
    };

    let direction = (target_position - current_position).as_vec2().normalize();
    let speed = calculate_movement_speed(monitor.size(), state.get());
    let delta = direction * speed * (time.delta_secs_f64() as f32);

    window.position = WindowPosition::At(current_position + delta.round().as_ivec2());
}

fn calculate_movement_speed(resolution: PhysicalSize<u32>, state: &BonnieState) -> f32 {
    let diagonal = ((resolution.width.pow(2) + resolution.height.pow(2)) as f32).sqrt();
    let base_speed = match state {
        BonnieState::Chasing => 2.0,
        _ => 1.0,
    };
    diagonal * 0.15 * base_speed
}

///////
// State-Specific Behaviour
///////

/////// Pooping

fn setup_poop_sprite(mut commands: Commands, asset_server: Res<AssetServer>) {
    // get the sprite
    let mut poop_sprite = Sprite::from_image(asset_server.load("BonPoop.png"));
    poop_sprite.custom_size = Some(Vec2::new(40.0, 40.0));

    // add to poop render layer
    commands.spawn((poop_sprite, RenderLayers::layer(POOP_LAYER)));
}

fn setup_pooping(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut machine: Query<&mut StateMachine>,
) {
    let window = window_query.single();

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
            PoopWindow,
        ))
        .id();

    commands.spawn((
        #[allow(deprecated)]
        Camera2dBundle {
            camera: Camera {
                target: RenderTarget::Window(WindowRef::Entity(poop_window)),
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(POOP_LAYER),
    ));

    machine.single_mut().finish();
}

/////// Chasing

fn handle_chasing(
    mut machine: Query<&mut StateMachine>,
    bonnie_query: Query<&mut Bonnie>,
    global_cursor_pos: Res<GlobalCursorPosition>,
    window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    let bonnie = bonnie_query.get_single().expect("Failed to get Bonnie.");
    if let BonnieState::Chasing = bonnie.state {
        // get window and machine
        let window = window_query.single();
        let mut machine = machine.single_mut();

        // get global cursor pos
        if let Some(cursor_pos) = global_cursor_pos.0 {
            // get bonnie position
            if let WindowPosition::At(bonnie_pos) = window.position {
                let diff = (bonnie_pos + IVec2::new(90, 147)).as_vec2() - cursor_pos;
                let dist = diff.length();

                // if cursor near bonnie, change state
                if dist < 35.0 {
                    info!("Close enough, finishing...");
                    machine.finish();
                }
            }
        }
    }
}

/////// Teaching

fn handle_teaching(
    mut teach_window: Query<&mut Window, (With<TeachWindow>, Without<PrimaryWindow>)>,
    bonnie_window: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    // get the teach window
    let Ok(mut window) = teach_window.get_single_mut() else {
        return;
    };

    // get bonnies position
    let bonnie_pos = match bonnie_window.single().position {
        WindowPosition::At(pos) => pos,
        _ => IVec2::ZERO,
    };

    let target = bonnie_pos + IVec2::new(-150, 200);

    // get the current teach position
    let current_pos = match window.position {
        WindowPosition::At(pos) => pos,
        _ => IVec2::ZERO,
    };

    // get direction and delta
    let direction = (target - current_pos).as_vec2().normalize();
    let delta = direction * 200.0 * (time.delta_secs_f64() as f32);

    // calculate remaining
    let remaining_vector = target - current_pos;
    let remaining_length = remaining_vector.as_vec2().length();
    let step_length = delta.length();

    // only step if needed
    if remaining_length <= step_length {
        window.position = WindowPosition::At(target);
    } else {
        window.position = WindowPosition::At(current_pos + delta.round().as_ivec2());
    }
}

fn setup_teaching(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<GlobalRng>,
    mut machine: Query<&mut StateMachine>,
) {
    info!("Blocking state machine...");
    machine.single_mut().block();

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
            TeachWindow,
        ))
        .id();

    // spawn a camera2d on TEACH_LAYER
    commands.spawn((
        #[allow(deprecated)]
        Camera2dBundle {
            camera: Camera {
                target: RenderTarget::Window(WindowRef::Entity(teach_window)),
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(TEACH_LAYER),
    ));
    // get the sprite
    let mut teach_sprite =
        Sprite::from_image(asset_server.load(random_education_image(&mut rng.0)));
    teach_sprite.custom_size = Some(Vec2::new(300.0, 300.0));

    // spawn the sprite on the render layer 1
    commands.spawn((teach_sprite, RenderLayers::layer(TEACH_LAYER)));
}

fn random_education_image(rng: &mut impl Rng) -> String {
    const IMAGES: &[&str] = &[
        "educational/meme1.png",
        "educational/meme2.png",
        "educational/meme3.png",
    ];
    IMAGES.choose(rng).unwrap().to_string()
}
