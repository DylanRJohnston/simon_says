use bevy::prelude::*;

use crate::level::GameFinished;

use super::{
    settings::CreateSettingsUI, PRIMARY_TEXT_COLOR, UI_BACKGROUND_COLOR, UI_CONTAINER_PADDING,
    UI_CONTAINER_RADIUS,
};

pub struct EndScreenPlugin;

impl Plugin for EndScreenPlugin {
    fn build(&self, app: &mut App) {
        app.observe(end_game).observe(despawn_endgame);
    }
}

#[derive(Debug, Component)]
pub struct EndScreenRoot;

fn end_game(_trigger: Trigger<GameFinished>, mut commands: Commands) {
    commands
        .spawn((NodeBundle {
            style: Style {
                height: Val::Percent(100.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        },EndScreenRoot))
        .with_children(|container| {
            container
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(500.),
                        padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                        ..default()
                    },
                    border_radius: BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
                    background_color: (*UI_BACKGROUND_COLOR).into(),
                    ..default()
                })
                .with_children(|container| {
                    container.spawn(TextBundle {
                        text: Text::from_section("That's all for now, check back later or at the end of the Jam for more content! Early feedback is greatly appreciated.", TextStyle {
                            color: *PRIMARY_TEXT_COLOR,
                            ..default()
                        }),
                        ..default()
                    });
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
