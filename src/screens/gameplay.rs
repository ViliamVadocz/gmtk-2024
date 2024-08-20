//! The screen state for the main gameplay.

use bevy::{prelude::*, ui::Val::*};

use crate::{
    asset_tracking::LoadResource,
    audio::Music,
    demo::{editor::EditorUI, level::spawn_level as spawn_level_command},
    screens::Screen,
    theme::palette::LABEL_TEXT,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_level);

    app.load_resource::<GameplayMusic>();
    app.add_systems(OnEnter(Screen::Gameplay), play_gameplay_music);
    app.add_systems(OnExit(Screen::Gameplay), stop_music);
}

#[derive(Component)]
pub struct AutoplayLabel;
impl AutoplayLabel {
    pub const DISABLED: &'static str = " (step F) (autoplay G) (respawn R)";
    pub const DISABLED_BIG: &'static str = "MANUAL MODE";
    pub const ENABLED: &'static str = "autoplay enabled (fast forward F) (manual G) (respawn R)";
}

#[derive(Component)]
pub struct UnlockedList;

fn spawn_level(mut commands: Commands) {
    commands.add(spawn_level_command);
    commands
        .spawn((Name::new("Gameplay UI Root"), NodeBundle {
            style: Style {
                width: Percent(100.0),
                height: Percent(100.0),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Px(10.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        }))
        .insert(StateScoped(Screen::Gameplay))
        .with_children(|children| {
            children.spawn((Name::new("Editor UI"), EditorUI, NodeBundle {
                style: Style {
                    width: Auto,
                    height: Percent(10.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                background_color: BackgroundColor(Color::hsl(0., 0., 0.9)),
                ..default()
            }));
            children
                .spawn(NodeBundle {
                    style: Style {
                        width: Percent(100.0),
                        height: Percent(100.0),
                        justify_content: JustifyContent::End,
                        align_items: AlignItems::Start,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|children| {
                    children
                        .spawn(NodeBundle {
                            style: Style {
                                width: Percent(100.0),
                                height: Percent(100.0),
                                justify_content: JustifyContent::Start,
                                align_items: AlignItems::End,
                                flex_direction: FlexDirection::Row,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|children| {
                            children.spawn((
                                AutoplayLabel,
                                TextBundle::from_section(AutoplayLabel::ENABLED, TextStyle {
                                    font_size: 24.0,
                                    color: LABEL_TEXT,
                                    ..default()
                                })
                                .with_no_wrap(),
                            ));
                            children.spawn((Name::new("Editor UI"), UnlockedList, NodeBundle {
                                style: Style {
                                    width: Percent(100.0),
                                    height: Percent(100.0),
                                    justify_content: JustifyContent::End,
                                    align_items: AlignItems::End,
                                    flex_direction: FlexDirection::Row,
                                    ..default()
                                },
                                ..default()
                            }));
                        });
                });
        });
}

#[derive(Resource, Asset, Reflect, Clone)]
pub struct GameplayMusic {
    #[dependency]
    handle: Handle<AudioSource>,
    entity: Option<Entity>,
}

impl FromWorld for GameplayMusic {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            handle: assets.load("audio/music/Fluffing A Duck.ogg"),
            entity: None,
        }
    }
}

fn play_gameplay_music(mut commands: Commands, mut music: ResMut<GameplayMusic>) {
    music.entity = Some(
        commands
            .spawn((
                AudioBundle {
                    source: music.handle.clone(),
                    settings: PlaybackSettings::LOOP,
                },
                Music,
            ))
            .id(),
    );
}

fn stop_music(mut commands: Commands, mut music: ResMut<GameplayMusic>) {
    if let Some(entity) = music.entity.take() {
        commands.entity(entity).despawn_recursive();
    }
}
