use std::time::Duration;

use bevy::window::WindowLevel;
use bevy::{prelude::*, window::CompositeAlphaMode};

mod plugins;
use plugins::bonnie_state;
use plugins::control;
use plugins::global_cursor;

pub mod bonnie;
use bonnie::{Bonnie, StateMachine};

#[cfg(target_os = "macos")]
pub fn get_composite_mode() -> CompositeAlphaMode {
    CompositeAlphaMode::PostMultiplied
}

#[cfg(not(target_os = "macos"))]
pub fn get_composite_mode() -> CompositeAlphaMode {
    CompositeAlphaMode::default()
}

#[cfg(target_os = "linux")]
fn configure_linux_audio() {
    unsafe {
        use std::path::PathBuf;

        // try to detect PulseAudio/PipeWire socket location
        let runtime_dir =
            std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());

        let pulse_path = PathBuf::from(&runtime_dir).join("pulse/native");
        let pipewire_path = PathBuf::from(&runtime_dir).join("pipewire-0");

        // try both PulseAudio and PipeWire sockets
        if pulse_path.exists() {
            std::env::set_var("PULSE_SERVER", pulse_path.to_str().unwrap());
        } else if pipewire_path.exists() {
            std::env::set_var("PULSE_SERVER", pipewire_path.to_str().unwrap());
        }

        // ALSA fallback configuration
        std::env::set_var("ALSA_DRIVER", "pulse");
        std::env::set_var("SDL_AUDIODRIVER", "pulse");
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    {
        configure_linux_audio();
        unsafe { std::env::set_var("BEVY_AUDIO_THREAD", "1") };
    }

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        transparent: true,
                        composite_alpha_mode: get_composite_mode(),
                        decorations: false,
                        resizable: false,
                        has_shadow: false,
                        titlebar_shown: false,
                        titlebar_transparent: false,
                        titlebar_show_buttons: false,
                        titlebar_show_title: false,
                        title: "Bonnie Buddy".to_string(),
                        name: Some("bonnie.buddy".into()),
                        resolution: (100.0, 100.0).into(),
                        resize_constraints: WindowResizeConstraints {
                            min_width: 100.0,
                            min_height: 100.0,
                            max_width: 100.0,
                            max_height: 100.0,
                        },
                        window_level: WindowLevel::AlwaysOnTop,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(control::BonnieControlPlugin)
        .add_plugins(bonnie_state::BonnieStatePlugin)
        .add_plugins(global_cursor::GlobalCursorPlugin)
        .insert_resource(ClearColor(Color::NONE))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let mut bonnie_sprite = Sprite::from_image(asset_server.load("BonNormal.png"));

    bonnie_sprite.custom_size = Some(Vec2::new(100.0, 100.0));

    commands.spawn((
        Bonnie::default(),
        Name::new("Bonnie"),
        StateMachine {
            timer: Timer::new(Duration::from_secs_f32(2.0), TimerMode::Once),
            can_change: true,
        },
        bonnie_sprite,
    ));
}
