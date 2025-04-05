use bevy::prelude::*;
use mouse_position::mouse_position::Mouse;

pub struct GlobalCursorPlugin;

impl Plugin for GlobalCursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalCursorPosition>()
            .add_systems(Update, track_global_cursor_position);
    }
}

#[derive(Resource, Default, Debug)]
pub struct GlobalCursorPosition(pub Option<Vec2>);

fn track_global_cursor_position(mut global_pos: ResMut<GlobalCursorPosition>) {
    let mouse = Mouse::get_mouse_position();
    if let Mouse::Position { x, y } = mouse {
        global_pos.0 = Some(Vec2::new(x as f32, y as f32));
    }
}
