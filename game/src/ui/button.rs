use bevy::prelude::*;

use super::{
    constants::{BUTTON_BORDER_THICKNESS, BUTTON_COLOR, PRIMARY_TEXT_COLOR},
    BUTTON_BORDER_RADIUS,
};

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, button_interaction);
    }
}

#[derive(Default)]
pub struct ButtonBuilder {
    on_click: Option<OnClick>,
    background_color: Option<Color>,
    border_color: Option<Color>,
    hover_background_color: Option<Color>,
    hover_border_color: Option<Color>,
    text: Option<String>,
}

impl ButtonBuilder {
    pub fn on_click(mut self, callback: OnClick) -> Self {
        self.on_click = Some(callback);
        self
    }

    pub fn text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn hover_background_color(mut self, color: Color) -> Self {
        self.hover_background_color = Some(color);
        self
    }

    pub fn hover_border_color(mut self, color: Color) -> Self {
        self.hover_border_color = Some(color);
        self
    }

    pub fn build(self, parent: &mut ChildBuilder) -> Entity {
        parent
            .spawn((
                Button {
                    on_click: self.on_click.unwrap(),
                    background_color: self.background_color.unwrap_or(*BUTTON_COLOR),
                    border_color: self.border_color.unwrap_or(*BUTTON_COLOR),
                    hover_background_color: self.hover_background_color.unwrap_or(*BUTTON_COLOR),
                    hover_border_color: self.hover_border_color.unwrap_or(*PRIMARY_TEXT_COLOR),
                },
                ButtonBundle {
                    style: Style {
                        align_self: AlignSelf::FlexStart,
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(BUTTON_BORDER_THICKNESS)),
                        ..default()
                    },
                    border_radius: BorderRadius::all(Val::Px(BUTTON_BORDER_RADIUS)),
                    background_color: (*BUTTON_COLOR).into(),
                    border_color: (*BUTTON_COLOR).into(),
                    ..default()
                },
            ))
            .with_children(|command_container| {
                command_container.spawn(TextBundle {
                    text: Text::from_section(
                        self.text.unwrap(),
                        TextStyle {
                            color: *PRIMARY_TEXT_COLOR,
                            ..default()
                        },
                    ),
                    ..default()
                });
            })
            .id()
    }
}

pub type OnClick = Box<dyn Fn(&mut Commands, Entity) + Send + Sync + 'static>;

#[derive(Component)]
pub struct Button {
    on_click: OnClick,
    background_color: Color,
    border_color: Color,
    hover_background_color: Color,
    hover_border_color: Color,
}

impl Button {
    pub fn builder() -> ButtonBuilder {
        ButtonBuilder::default()
    }
}

fn button_interaction(
    mut commands: Commands,
    mut actions: Query<
        (
            Entity,
            &Interaction,
            &Button,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
) {
    for (entity, interaction, button, mut button_color, mut background_color) in &mut actions {
        match interaction {
            Interaction::Pressed => {
                *button_color = BorderColor::from(button.hover_border_color);
                *background_color = BackgroundColor::from(button.hover_border_color);

                (button.on_click)(&mut commands, entity);
            }
            Interaction::Hovered => {
                *button_color = BorderColor::from(button.hover_border_color);
                *background_color = BackgroundColor::from(button.hover_background_color);
            }
            Interaction::None => {
                *button_color = BorderColor::from(button.border_color);
                *background_color = BackgroundColor::from(button.background_color);
            }
        }
    }
}
