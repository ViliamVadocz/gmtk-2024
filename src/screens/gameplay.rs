//! The screen state for the main gameplay.

use bevy::{input::common_conditions::input_just_pressed, prelude::*, ui::Val::*};
use bevy_simple_text_input::{TextInputInactive, TextInputSubmitEvent};

use crate::{
    asset_tracking::LoadResource,
    audio::Music,
    demo::{action::ScriptCommand, level::spawn_level as spawn_level_command, player::PlayerState},
    screens::Screen,
    theme::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_level);

    app.load_resource::<GameplayMusic>();
    app.add_systems(OnEnter(Screen::Gameplay), play_gameplay_music);
    app.add_systems(OnExit(Screen::Gameplay), stop_music);

    app.add_systems(
        Update,
        return_to_title_screen
            .run_if(in_state(Screen::Gameplay).and_then(input_just_pressed(KeyCode::Escape))),
    );

    app.add_systems(Update, text_input_listener);
}

#[derive(Component)]
pub struct Editor;

fn spawn_level(mut commands: Commands) {
    commands.add(spawn_level_command);
    commands
        .spawn((Name::new("UI Root"), NodeBundle {
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
            children.text_input();
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

fn return_to_title_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn text_input_listener(
    mut events: EventReader<TextInputSubmitEvent>,
    mut player_query: Query<&mut PlayerState>,
    mut editor_inactive: Query<&mut TextInputInactive, With<Editor>>,
) {
    for event in events.read() {
        for mut player_state in &mut player_query {
            let new_sequence: Vec<_> = event
                .value
                .chars()
                .filter_map(ScriptCommand::try_from)
                .collect();
            if new_sequence.is_empty() {
                continue;
            }

            editor_inactive.single_mut().0 = true;
            player_state.sequence = new_sequence;
            player_state.cursor = 0;
        }
    }
}
