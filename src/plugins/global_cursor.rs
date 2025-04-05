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
    use core_graphics::display::{CGDisplay, CGMainDisplayID};
    use core_graphics::event::{CGEvent, CGEventType};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    // Get mouse location in screen coordinates
    let point = unsafe {
        let event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .expect("Failed to create event source");
        CGEvent::new(event_source).location()
    };

    // Get display dimensions for coordinate conversion
    let main_display = unsafe { CGDisplay::new(CGMainDisplayID()) };
    let screen_height = main_display.pixels_high() as f32;

    // Convert to top-left origin coordinates
    global_pos.0 = Some(Vec2::new(
        point.x as f32,
        screen_height - point.y as f32, // Flip Y axis
    ));
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
