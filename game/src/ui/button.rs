use bevy::{ecs::system::EntityCommands, prelude::*};

use super::{
    constants::{BUTTON_BORDER_THICKNESS, BUTTON_COLOR, PRIMARY_TEXT_COLOR},
    BUTTON_BORDER_RADIUS, GHOST_TEXT_COLOR, UI_BACKGROUND_COLOR,
};

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Button>()
            .add_systems(Update, button_interaction)
            .add_systems(Update, update_style);
    }
}

#[derive(Default)]
pub struct ButtonBuilder {
    on_click: Option<Box<OnClick>>,
    background_color: Option<Color>,
    border_color: Option<Color>,
    hover_background_color: Option<Color>,
    hover_border_color: Option<Color>,
    text_color: Option<Color>,
    text: Option<String>,
    icon: Option<Handle<Image>>,
    size: Option<f32>,
    disabled: bool,
}

impl ButtonBuilder {
    pub fn on_click(mut self, callback: impl Fn(&mut Commands) + Send + Sync + 'static) -> Self {
        self.on_click = Some(Box::new(callback));
        self
    }

    pub fn text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    pub fn icon(mut self, icon: Handle<Image>) -> Self {
        self.icon = Some(icon);
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

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    pub fn build<'a>(self, parent: &'a mut ChildBuilder) -> EntityCommands<'a> {
        if self.on_click.is_none() {
            panic!("Button must have an on_click callback.");
        }

        let mut commands = parent.spawn((
            Name::from("Button"),
            Button {
                on_click: self.on_click.unwrap(),
                background_color: self.background_color,
                border_color: self.border_color,
                hover_background_color: self.hover_background_color,
                hover_border_color: self.hover_border_color,
                text_color: self.text_color,
                disabled: self.disabled,
            },
            ButtonBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(BUTTON_BORDER_THICKNESS)),
                    ..default()
                },
                border_radius: BorderRadius::all(Val::Px(BUTTON_BORDER_RADIUS)),
                background_color: self.background_color.unwrap_or(*BUTTON_COLOR).into(),
                border_color: self
                    .border_color
                    .unwrap_or(self.background_color.unwrap_or(*BUTTON_COLOR))
                    .into(),
                ..default()
            },
        ));
        commands.with_children(|command_container| {
            if let Some(text) = self.text {
                command_container.spawn(TextBundle {
                    text: Text::from_section(
                        text,
                        TextStyle {
                            color: self.text_color.unwrap_or(*PRIMARY_TEXT_COLOR),
                            ..default()
                        },
                    ),
                    ..default()
                });
            }
            if let Some(icon) = self.icon {
                let size = self.size.unwrap_or(20.);

                command_container.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(size),
                            height: Val::Px(size),
                            ..default()
                        },
                        ..default()
                    },
                    UiImage::new(icon),
                ));
            }
        });
        commands
    }
}

pub type OnClick = dyn Fn(&mut Commands) + Send + Sync + 'static;

#[derive(Component, Reflect)]
#[reflect(from_reflect = false)]
pub struct Button {
    #[reflect(ignore)]
    pub on_click: Box<OnClick>,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub hover_background_color: Option<Color>,
    pub hover_border_color: Option<Color>,
    pub text_color: Option<Color>,
    pub disabled: bool,
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
            &Interaction,
            &Button,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
) {
    for (interaction, button, mut border_color, mut background_color) in &mut actions {
        if button.disabled {
            *border_color = BorderColor::from(*UI_BACKGROUND_COLOR);
            *background_color = BackgroundColor::from(*UI_BACKGROUND_COLOR);

            continue;
        }

        match interaction {
            Interaction::Pressed => {
                *border_color = button
                    .hover_border_color
                    .unwrap_or(*PRIMARY_TEXT_COLOR)
                    .into();

                *background_color = button
                    .hover_border_color
                    .unwrap_or(*PRIMARY_TEXT_COLOR)
                    .into();

                (button.on_click)(&mut commands);
            }
            Interaction::Hovered => {
                *border_color = button
                    .hover_border_color
                    .unwrap_or(*PRIMARY_TEXT_COLOR)
                    .into();

                *background_color = button
                    .hover_background_color
                    .unwrap_or(button.background_color.unwrap_or(*BUTTON_COLOR))
                    .into();
            }
            Interaction::None => {
                *border_color = button
                    .border_color
                    .unwrap_or(button.background_color.unwrap_or(*BUTTON_COLOR))
                    .into();
                *background_color = button.background_color.unwrap_or(*BUTTON_COLOR).into();
            }
        }
    }
}

fn update_style(
    mut query: Query<(&mut BorderColor, &mut BackgroundColor, &Button, &Children), Changed<Button>>,
    mut text: Query<&mut Text>,
) {
    for (mut border_color, mut background_color, button, children) in &mut query {
        if let Ok(mut text) = text.get_mut(children[0]) {
            for section in &mut text.sections {
                section.style.color = if !button.disabled {
                    button.text_color.unwrap_or(*PRIMARY_TEXT_COLOR)
                } else {
                    *GHOST_TEXT_COLOR
                };
            }
        }

        if button.disabled {
            *border_color = BorderColor::from(*UI_BACKGROUND_COLOR);
            *background_color = BackgroundColor::from(*UI_BACKGROUND_COLOR);

            continue;
        }

        tracing::info!("setting undisabled style");

        *background_color = button.background_color.unwrap_or(*BUTTON_COLOR).into();
        *border_color = button
            .border_color
            .unwrap_or(button.background_color.unwrap_or(*BUTTON_COLOR))
            .into();
    }
}
