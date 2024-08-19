//! Helper traits for creating common widgets.

use bevy::{ecs::system::EntityCommands, prelude::*, ui::Val::*};
use bevy_simple_text_input::{TextInputBundle, TextInputSettings};

use crate::theme::{interaction::InteractionPalette, palette::*};

/// An extension trait for spawning UI widgets.
pub trait Widgets {
    /// Spawn a simple button with text.
    fn button(&mut self, text: impl Into<String>) -> EntityCommands;

    /// Spawn a simple header label. Bigger than [`Widgets::label`].
    fn header(&mut self, text: impl Into<String>) -> EntityCommands;

    /// Spawn a simple text label.
    fn label(&mut self, text: impl Into<String>) -> EntityCommands;

    /// Spawn a text input.
    fn text_input(&mut self) -> EntityCommands;
}

impl<T: Spawn> Widgets for T {
    fn button(&mut self, text: impl Into<String>) -> EntityCommands {
        let mut entity = self.spawn((
            Name::new("Button"),
            ButtonBundle {
                style: Style {
                    width: Px(200.0),
                    height: Px(65.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(NODE_BACKGROUND),
                ..default()
            },
            InteractionPalette {
                none: NODE_BACKGROUND,
                hovered: BUTTON_HOVERED_BACKGROUND,
                pressed: BUTTON_PRESSED_BACKGROUND,
            },
        ));
        entity.with_children(|children| {
            children.spawn((
                Name::new("Button Text"),
                TextBundle::from_section(text, TextStyle {
                    font_size: 40.0,
                    color: BUTTON_TEXT,
                    ..default()
                }),
            ));
        });

        entity
    }

    fn header(&mut self, text: impl Into<String>) -> EntityCommands {
        let mut entity = self.spawn((Name::new("Header"), NodeBundle {
            style: Style {
                width: Px(500.0),
                height: Px(65.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(NODE_BACKGROUND),
            ..default()
        }));
        entity.with_children(|children| {
            children.spawn((
                Name::new("Header Text"),
                TextBundle::from_section(text, TextStyle {
                    font_size: 40.0,
                    color: HEADER_TEXT,
                    ..default()
                }),
            ));
        });
        entity
    }

    fn label(&mut self, text: impl Into<String>) -> EntityCommands {
        let entity = self.spawn((
            Name::new("Label"),
            TextBundle::from_section(text, TextStyle {
                font_size: 24.0,
                color: LABEL_TEXT,
                ..default()
            })
            .with_style(Style {
                width: Px(500.0),
                ..default()
            }),
        ));
        entity
    }

    fn text_input(&mut self) -> EntityCommands {
        let entity = self.spawn((
            Name::new("Text Input"),
            NodeBundle {
                style: Style {
                    width: Val::Px(200.0),
                    border: UiRect::all(Val::Px(5.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                border_color: BUTTON_PRESSED_BACKGROUND.into(),
                background_color: BUTTON_HOVERED_BACKGROUND.into(),
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            },
            TextInputBundle::default()
                .with_text_style(TextStyle {
                    font_size: 24.,
                    color: LABEL_TEXT,
                    ..default()
                })
                .with_settings(TextInputSettings {
                    retain_on_submit: true,
                    ..default()
                }),
        ));

        entity
    }
}

/// An extension trait for spawning UI containers.
pub trait Containers {
    /// Spawns a root node that covers the full screen
    /// and centers its content horizontally and vertically.
    fn ui_root(&mut self) -> EntityCommands;
}

impl Containers for Commands<'_, '_> {
    fn ui_root(&mut self) -> EntityCommands {
        self.spawn((Name::new("UI Root"), NodeBundle {
            style: Style {
                width: Percent(100.0),
                height: Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Px(10.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        }))
    }
}

/// An internal trait for types that can spawn entities.
/// This is here so that [`Widgets`] can be implemented on all types that
/// are able to spawn entities.
/// Ideally, this trait should be [part of Bevy itself](https://github.com/bevyengine/bevy/issues/14231).
trait Spawn {
    fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands;
}

impl Spawn for Commands<'_, '_> {
    fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands {
        self.spawn(bundle)
    }
}

impl Spawn for ChildBuilder<'_> {
    fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands {
        self.spawn(bundle)
    }
}
