use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        transparent: true,
                        decorations: false,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
        )
        .insert_resource(ClearColor(Color::NONE))
        .add_systems(Startup, setup)
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
