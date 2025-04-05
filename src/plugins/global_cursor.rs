use bevy::prelude::*;
use enigo::{Enigo, Mouse, Settings};

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

fn track_global_cursor_position(mut global_pos: ResMut<GlobalCursorPosition>) {
    let enigo = Enigo::new(&Settings::default()).expect("Couldn't make Enigo.");

    if let Ok(position) = enigo.location() {
        global_pos.0 = Some(Vec2::new(position.0 as f32, position.1 as f32));
    }
}

fn print_global_cursor_position(global_pos: Res<GlobalCursorPosition>) {
    info!("Position: {:?}", global_pos.0);
}
