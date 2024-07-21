use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;
use game::{actions::ActionPlugin, level::LevelPlugin, player::PlayerPlugin, ui::UIPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(LevelPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(ActionPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(TweeningPlugin)
        .insert_resource(ClearColor(Color::srgb_u8(0x33, 0x3c, 0x57)))
        .insert_resource(AmbientLight {
            brightness: 80.0,
            color: Color::WHITE,
        })
        .add_systems(Startup, setup);

    #[cfg(feature = "debug")]
    app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

    app.run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands) {
    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 8_000.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::default(),
            rotation: Quat::from_euler(EulerRot::XYZ, -1.1, 0.5, 0.0),
            scale: Vec3::ONE,
        },
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        IsDefaultUiCamera,
    ));
}
