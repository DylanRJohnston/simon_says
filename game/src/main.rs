use bevy::asset::{AssetMetaCheck, load_internal_binary_asset};
use bevy::core_pipeline::fxaa::Fxaa;
use bevy::prelude::*;
use bevy_firework::plugin::ParticleSystemPlugin;
use bevy_tweening::TweeningPlugin;
use game::assets::AssetsPlugin;
use game::game_state::GameStatePlugin;
use game::music::MusicPlugin;
use game::video_glitch::VideoGlitchPlugin;
use game::{
    actions::ActionPlugin, delayed_command::DelayedCommandPlugin, eyes::EyesPlugin,
    level::LevelPlugin, player::PlayerPlugin, simulation::SimulationPlugin, ui::UIPlugin,
};

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Simon Says".into(),
                    canvas: Some("#bevy".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .add_plugins(LevelPlugin)
    .add_plugins(PlayerPlugin)
    .add_plugins(ActionPlugin)
    .add_plugins(UIPlugin)
    .add_plugins(TweeningPlugin)
    .add_plugins(SimulationPlugin)
    .add_plugins(DelayedCommandPlugin)
    .add_plugins(GameStatePlugin)
    .add_plugins(EyesPlugin)
    .add_plugins(ParticleSystemPlugin::default())
    .add_plugins(MusicPlugin)
    .add_plugins(VideoGlitchPlugin)
    .add_plugins(AssetsPlugin)
    // .insert_resource(ClearColor(Color::srgb_u8(0x33, 0x3c, 0x57)))
    .insert_resource(ClearColor(Color::srgb_u8(0xdd, 0xdd, 0xdd)))
    .insert_resource(AmbientLight {
        brightness: 80.0,
        color: Color::WHITE,
        ..Default::default()
    })
    .add_systems(Startup, setup);

    #[cfg(feature = "debug")]
    app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

    load_internal_binary_asset!(
        app,
        TextFont::default().font,
        // "../assets/fonts/Neuropol X Rg.otf",
        "../assets/fonts/LEMONMILK-Regular.otf",
        |bytes: &[u8], _path: String| { Font::try_from_bytes(bytes.to_vec()).unwrap() }
    );

    app.run();
}

fn setup(mut commands: Commands) {
    // light
    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::default(),
            rotation: Quat::from_euler(EulerRot::XYZ, -1.1, 0.5, 0.0),
            scale: Vec3::ONE,
        },
    ));

    let mut transform = Transform::from_xyz(-3., 5.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y);
    transform.translation += Vec3::Y * 1.0;

    // camera
    commands.spawn((
        Camera3d::default(),
        transform,
        Camera {
            hdr: false,
            ..default()
        },
        IsDefaultUiCamera,
        Fxaa::default(),
        bevy_video_glitch::VideoGlitchSettings {
            intensity: 0.,
            ..default()
        },
        #[cfg(target_arch = "wasm32")]
        Msaa::Off,
    ));
}
