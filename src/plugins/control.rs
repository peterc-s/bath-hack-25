//! Keyboard controls for debugging.
//!
//! Arrow keys move the window, q will quit.

use bevy::{prelude::*, window::PrimaryWindow};

pub struct BonnieControlPlugin;

impl Plugin for BonnieControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (move_window, quit_on_q));
    }
}

fn move_window(
    key_input: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    // get window
    if let Ok(mut window) = window_query.get_single_mut() {
        // pixels/frame
        let move_speed = 10;

        // get current window position
        let current_pos = match window.position {
            WindowPosition::At(pos) => pos,
            _ => IVec2::new(100, 100),
        };

        // get new position
        let mut new_pos = current_pos;
        if key_input.pressed(KeyCode::ArrowLeft) {
            new_pos.x -= move_speed;
        }
        if key_input.pressed(KeyCode::ArrowRight) {
            new_pos.x += move_speed;
        }
        if key_input.pressed(KeyCode::ArrowUp) {
            new_pos.y -= move_speed;
        }
        if key_input.pressed(KeyCode::ArrowDown) {
            new_pos.y += move_speed;
        }

        // update the position
        window.position = WindowPosition::At(new_pos);
    }
}

fn quit_on_q(
    key_input: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<bevy::app::AppExit>,
) {
    if key_input.just_pressed(KeyCode::KeyQ) {
        app_exit_events.send(AppExit::Success);
    }
}
