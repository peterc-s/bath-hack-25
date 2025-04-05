use bevy::prelude::*;
use mouse_position::mouse_position::Mouse;

pub struct GlobalCursorPlugin;

impl Plugin for GlobalCursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalCursorPosition>()
            // .add_systems(Update, print_global_cursor_position)
            .add_systems(Update, track_global_cursor_position);
    }
}

#[derive(Resource, Default, Debug)]
pub struct GlobalCursorPosition(pub Option<Vec2>);

#[cfg(target_os = "macos")]
fn track_global_cursor_position(mut global_pos: ResMut<GlobalCursorPosition>) {
    let mouse = Mouse::get_mouse_position();

    if let Mouse::Position { x, y } = mouse {
        global_pos.0 = Some(Vec2::new(x * 2 as f32, y * 2 as f32));
    }
}

#[cfg(not(target_os = "macos"))]
fn track_global_cursor_position(mut global_pos: ResMut<GlobalCursorPosition>) {
    let mouse = Mouse::get_mouse_position();

    if let Mouse::Position { x, y } = mouse {
        global_pos.0 = Some(Vec2::new(x as f32, y as f32));
    }
}

fn print_global_cursor_position(global_pos: Res<GlobalCursorPosition>) {
    info!("Position: {:?}", global_pos.0);
}
