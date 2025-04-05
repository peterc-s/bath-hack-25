use bevy::{prelude::*, window::PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        transparent: true,
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
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
        )
        .insert_resource(ClearColor(Color::NONE))
        .add_systems(Startup, setup)
        .add_systems(Update, move_window)
        .add_systems(Update, quit_on_q)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d::default());

    let mut bonnie_sprite = Sprite::from_image(
        asset_server.load("bonnietest.png")
    );

    bonnie_sprite.custom_size = Some(Vec2::new(100.0, 100.0));

    commands.spawn(
        bonnie_sprite
    );
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
            _ => IVec2::new(100, 100)
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
