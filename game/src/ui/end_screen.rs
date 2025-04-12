use bevy::prelude::*;

use crate::level::GameFinished;

use super::{
    settings::CreateSettingsUI, PRIMARY_TEXT_COLOR, UI_BACKGROUND_COLOR, UI_CONTAINER_PADDING,
    UI_CONTAINER_RADIUS,
};

pub struct EndScreenPlugin;

impl Plugin for EndScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(end_game).add_observer(despawn_endgame);
    }
}

#[derive(Debug, Component)]
pub struct EndScreenRoot;

fn end_game(_trigger: Trigger<GameFinished>, mut commands: Commands) {
    commands
        .spawn((
            Name::new("End Screen Root"),
            EndScreenRoot,
            Node {
                height: Val::Percent(100.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|container| {
            container
                .spawn((
                    Node {
                        width: Val::Px(500.),
                        padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
                    BackgroundColor((*UI_BACKGROUND_COLOR).into()),
                ))
                .with_children(|container| {
                    container.spawn((
                        Text(
                            "That's all Simon has to say for now. Thanks for playing and keep an EYE out on Steam for the full release.".into()
                        ),
                            TextColor(*PRIMARY_TEXT_COLOR),)
                    );
                });
        });
}

fn despawn_endgame(
    _trigger: Trigger<CreateSettingsUI>,
    mut commands: Commands,
    end_screen: Query<Entity, With<EndScreenRoot>>,
) {
    for entity in &end_screen {
        commands.entity(entity).despawn_recursive();
    }
}
