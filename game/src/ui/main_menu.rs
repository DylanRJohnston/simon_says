use bevy::prelude::*;

use super::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup)
            .observe(start_game)
            .add_systems(OnExit(GameState::MainMenu), destroy);
    }
}

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Debug, Event)]
pub struct StartGame;

fn setup(mut commands: Commands) {
    commands
        .spawn((
            MainMenuRoot,
            NodeBundle {
                style: Style {
                    height: Val::Percent(100.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|container| {
            button::Button::builder()
                .text("Begin".into())
                .on_click(Box::new(|commands, _| {
                    commands.trigger(StartGame);
                }))
                .build(container);
        });
}

fn destroy(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn start_game(_trigger: Trigger<StartGame>, mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::InGame);
}
