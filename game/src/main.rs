use std::time::Duration;

use bevy::prelude::*;
use bevy_firework::plugin::ParticleSystemPlugin;
use bevy_kira_audio::prelude::*;
use bevy_kira_audio::{Audio, AudioPlugin};
use bevy_tweening::TweeningPlugin;
use game::game_state::GameStatePlugin;
use game::music::MusicPlugin;
use game::{
    actions::ActionPlugin, delayed_command::DelayedCommandPlugin, eyes::EyesPlugin,
    level::LevelPlugin, player::PlayerPlugin, simulation::SimulationPlugin, ui::UIPlugin,
};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(LevelPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(ActionPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(TweeningPlugin)
        .add_plugins(SimulationPlugin)
        .add_plugins(DelayedCommandPlugin)
        .add_plugins(GameStatePlugin)
        .add_plugins(EyesPlugin)
        .add_plugins(ParticleSystemPlugin)
        .add_plugins(MusicPlugin)
        // .insert_resource(ClearColor(Color::srgb_u8(0x33, 0x3c, 0x57)))
        .insert_resource(ClearColor(Color::srgb_u8(0xdd, 0xdd, 0xdd)))
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

    let mut transform = Transform::from_xyz(-3., 5.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y);
    transform.translation += Vec3::Y * 1.0;

    // camera
    commands.spawn((
        Camera3dBundle {
            transform,
            ..default()
        },
        IsDefaultUiCamera,
    ));
}
