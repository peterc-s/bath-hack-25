use std::time::Duration;

use bevy::{prelude::*, window::CompositeAlphaMode};

mod plugins;
use plugins::bonnie_state;
use plugins::control;

pub mod bonnie;
use bonnie::{Bonnie, BonnieState, StateMachine};

#[cfg(target_os = "macos")]
fn get_composite_mode() -> CompositeAlphaMode {
    CompositeAlphaMode::PostMultiplied
}

#[cfg(not(target_os = "macos"))]
fn get_composite_mode() -> CompositeAlphaMode {
    CompositeAlphaMode::default()
}

fn main() {
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
                        resolution: (50.0, 50.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(control::BonnieControlPlugin)
        .add_plugins(bonnie_state::BonnieStatePlugin)
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
