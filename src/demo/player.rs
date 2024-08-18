//! Plugin handling the player character in particular.
//! Note that this is separate from the `movement` module as that could be used
//! for other characters as well.

use bevy::{prelude::*, sprite::Anchor};
use bevy_simple_text_input::TextInputInactive;

use super::{
    action::ScriptCommand,
    animation::{AnimationState, PlayerAssets},
    level::{GridTransform, Level},
};
use crate::{
    asset_tracking::LoadResource,
    screens::{gameplay::Editor, Screen},
    AppSet,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.load_resource::<PlayerAssets>();
    app.init_resource::<Script>();
    app.init_resource::<Paused>();
    app.add_systems(
        Update,
        (
            (action_interpreter, respawn)
                .chain()
                .in_set(AppSet::DetermineAction),
            // record_player_directional_input.in_set(AppSet::DetermineAction),
            camera_follow_player.in_set(AppSet::UpdateCamera),
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}

fn action_interpreter(
    mut player: Query<(&GridTransform, &mut AnimationState), With<Player>>,
    mut script: ResMut<Script>,
    level: Res<Level>,
    assets: Res<PlayerAssets>,
) {
    let Ok((pos, mut state)) = player.get_single_mut() else {
        return;
    };
    if script.sequence.is_empty() {
        state.animation = None;
        return;
    }

    // Rename for convenience.
    let sequence = &script.sequence;
    let mut cursor = script.cursor;

    // Helper functions to clean up the interpreter code below.
    let possible = |command: ScriptCommand| -> bool {
        let anim = command.get_resource(&assets);
        level.animation_is_valid(pos.0, anim, state.x_dir)
    };
    let find_matching_open_bracket = |cursor| {
        let mut count = 0;
        for i in 1..=cursor {
            match sequence[cursor - i] {
                ScriptCommand::CloseBracket => count += 1,
                ScriptCommand::OpenBracket if count == 0 => {
                    return cursor - i;
                }
                ScriptCommand::OpenBracket => count -= 1,
                _ => {}
            }
        }
        0
    };
    let find_matching_close_bracket = |cursor| {
        let mut count = 0;
        for (i, cmd) in sequence.iter().enumerate().skip(cursor) {
            match cmd {
                ScriptCommand::OpenBracket => count += 1,
                ScriptCommand::CloseBracket if count == 0 => {
                    return i;
                }
                ScriptCommand::CloseBracket => count -= 1,
                _ => {}
            }
        }
        sequence.len() - 1
    };

    // Prevent infinite loops by limiting the number of iterations.
    for _ in 0..sequence.len() {
        match sequence[cursor] {
            ScriptCommand::OpenBracket => {}
            ScriptCommand::CloseBracket => {
                // Go back to matching open bracket.
                cursor = find_matching_open_bracket(cursor);
            }
            command if possible(command) => {
                // Update the cursor.
                script.cursor = (cursor + 1) % sequence.len();
                // Set the animation.
                let anim = command.get_resource(&assets);
                if let ScriptCommand::Turn = command {
                    state.x_dir *= -1
                };
                state.animation = Some(anim.clone());
                return;
            }
            _command => {
                // Skip to the end of scope.
                cursor = find_matching_close_bracket(cursor);
            }
        }
        // Try the next command.
        cursor += 1;
        cursor %= sequence.len();
    }

    // No action from the script was possible.
    state.animation = None;
}

fn respawn(
    input: Res<ButtonInput<KeyCode>>,
    level: Res<Level>,
    mut paused: ResMut<Paused>,
    mut script: ResMut<Script>,
    mut player: Query<(&mut GridTransform, &mut AnimationState)>,
    mut editor_inactive: Query<&mut TextInputInactive, With<Editor>>,
) {
    let Ok((mut pos, mut state)) = player.get_single_mut() else {
        return;
    };
    if input.just_pressed(KeyCode::KeyR) {
        // respawn, reset all properties
        pos.0 = level.last_checkpoint; // TODO: decouple last checkpoint from level resource?
        state.x_dir = 1;
        paused.0 = true;
        state.animation = None;
        script.cursor = 0;
        // allow editing again
        editor_inactive.single_mut().0 = false;
    }
}

/// Marker component for the player.
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Player;

/// The state of the robot script.
#[derive(Resource)]
pub struct Script {
    pub sequence: Vec<ScriptCommand>,
    pub cursor: usize,
}

impl Default for Script {
    fn default() -> Self {
        Self {
            sequence: vec![ScriptCommand::Walk, ScriptCommand::Climb],
            cursor: 0,
        }
    }
}

/// Whether the script execution is paused.
#[derive(Resource)]
pub struct Paused(pub bool);

impl Default for Paused {
    fn default() -> Self {
        Self(true)
    }
}

pub fn spawn_player(mut commands: Commands, player_assets: Res<PlayerAssets>, level: Res<Level>) {
    commands.spawn((
        Name::new("Player"),
        Player,
        SpriteBundle {
            texture: player_assets.idle.texture.clone(),
            transform: Transform::from_scale(Vec2::splat(4.0).extend(1.0)),
            sprite: Sprite {
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            ..Default::default()
        },
        GridTransform(level.get_spawn()),
        AnimationState {
            x_dir: 1,
            animation: None,
        },
        TextureAtlas {
            layout: player_assets.idle.atlas.clone(),
            index: 0,
        },
        StateScoped(Screen::Gameplay),
    ));
}

fn camera_follow_player(
    mut camera: Query<&mut Transform, With<IsDefaultUiCamera>>,
    player: Query<&Transform, (With<Player>, Without<IsDefaultUiCamera>)>,
    time: Res<Time>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };
    for mut camera in &mut camera {
        let target = player.translation.xy().extend(camera.translation.z);
        const SPEED: f32 = 0.9;
        let old_part = (1. - SPEED).powf(time.delta_seconds());
        camera.translation = target.lerp(camera.translation, old_part);
    }
}
