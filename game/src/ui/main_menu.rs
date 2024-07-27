use bevy::prelude::*;

use crate::delayed_command::DelayedCommand;

use super::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup)
            .observe(start_game)
            .observe(remove_ui)
            .observe(spawn_main_menu)
            .observe(refuse)
            .add_systems(OnExit(GameState::MainMenu), destroy);
    }
}

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Debug, Event)]
pub struct StartGame;

fn setup(mut commands: Commands) {
    commands.trigger(SpawnMainMenu);
}

#[derive(Debug, Event)]
struct SpawnMainMenu;

fn spawn_main_menu(_trigger: Trigger<SpawnMainMenu>, mut commands: Commands) {
    commands.spawn(DelayedCommand::new(0.5, |commands| {
        commands
            .spawn((
                MainMenuRoot,
                NodeBundle {
                    style: Style {
                        height: Val::Percent(100.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        column_gap: Val::Px(UI_CONTAINER_GAP),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|container| {
                button::Button::builder()
                    .text("Begin Cycle No. 4,815,162,342".into())
                    .on_click(Box::new(|commands, _| {
                        commands.trigger(StartGame);
                    }))
                    .build(container);
                button::Button::builder()
                    .text("Disobey".into())
                    // .text("Vibe".into())
                    .background_color(*BUTTON_CANCEL_COLOR)
                    .on_click(Box::new(|commands, _| {
                        commands.trigger(RemoveUI);
                        commands.trigger(Refuse);
                    }))
                    .build(container);
            });
    }));
}

fn destroy(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    commands.trigger(RemoveUI);
}

#[derive(Debug, Event)]
pub struct RemoveUI;

#[derive(Debug, Event)]
pub struct Refuse;

fn refuse(
    _trigger: Trigger<Refuse>,
    mut commands: Commands,
    query: Query<Entity, With<MainMenuRoot>>,
) {
    commands.spawn(DelayedCommand::new(35., |commands| {
        commands.trigger(SpawnMainMenu);
    }));
}

fn remove_ui(
    _trigger: Trigger<RemoveUI>,
    mut commands: Commands,
    query: Query<Entity, With<MainMenuRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn start_game(_trigger: Trigger<StartGame>, mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::InGame);
}
