//! Plugin handling the player character in particular.
//! Note that this is separate from the `movement` module as that could be used
//! for other characters as well.

use std::time::Duration;

use bevy::{
    ecs::{system::RunSystemOnce as _, world::Command},
    prelude::*,
    sprite::Anchor,
};
use bevy_simple_text_input::TextInputInactive;

use super::{
    action::ScriptCommand,
    animation::{AnimationResource, PlayerAssets},
    level::{AnimationTick, GridTransform, Level, NextTick},
};
use crate::{
    asset_tracking::LoadResource,
    screens::{gameplay::Editor, Screen},
    AppSet,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            respawn,
            action_interpreter.in_set(AppSet::RecordInput),
            camera_follow_player.in_set(AppSet::UpdateCamera),
        ),
    );
    app.insert_resource(PlayerState {
        x_dir: 1,
        animation: None,
        sequence: vec![ScriptCommand::Walk, ScriptCommand::Climb],
        cursor: 0,
        autoplay: true,
    });
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

/// A command to spawn the player character.
#[derive(Debug)]
pub struct SpawnPlayer;

impl Command for SpawnPlayer {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self, spawn_player);
    }
}

#[derive(Resource)]
pub struct PlayerState {
    // can be 1 or -1
    pub x_dir: i32,
    pub animation: Option<AnimationResource>,

    pub sequence: Vec<ScriptCommand>,
    pub cursor: usize,
    pub autoplay: bool,
}

fn spawn_player(
    In(_config): In<SpawnPlayer>,
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    level: Res<Level>,
) {
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
        TextureAtlas {
            layout: player_assets.idle.atlas.clone(),
            index: 0,
        },
        StateScoped(Screen::Gameplay),
    ));
}

fn debug_actions(input: &ButtonInput<KeyCode>, state: &mut PlayerState) -> Option<ScriptCommand> {
    let pressed_or_held = |key: KeyCode| input.pressed(key);

    // Collect directional input.
    let mut action = None;

    let mut facing = 0;
    if pressed_or_held(KeyCode::KeyA) || pressed_or_held(KeyCode::ArrowLeft) {
        facing -= 1;
    }
    if pressed_or_held(KeyCode::KeyD) || pressed_or_held(KeyCode::ArrowRight) {
        facing += 1;
    }
    if facing != 0 {
        if state.x_dir != facing {
            return Some(ScriptCommand::Turn);
        }
        action = Some(ScriptCommand::Walk)
    }
    if pressed_or_held(KeyCode::KeyW) || pressed_or_held(KeyCode::ArrowUp) {
        action = Some(ScriptCommand::Climb)
    }
    if pressed_or_held(KeyCode::KeyS) || pressed_or_held(KeyCode::ArrowDown) {
        action = Some(ScriptCommand::Drop)
    }
    if pressed_or_held(KeyCode::Space) {
        action = Some(ScriptCommand::Idle)
    }
    action
}

fn respawn(
    mut state: ResMut<PlayerState>,
    mut player: Query<&mut GridTransform, With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    level: Res<Level>,
    mut editor_inactive: Query<&mut TextInputInactive, With<Editor>>,
) {
    let Ok(mut pos) = player.get_single_mut() else {
        return;
    };

    if input.just_pressed(KeyCode::KeyR) {
        // respawn, reset all properties
        pos.0 = level.last_checkpoint;
        state.x_dir = 1;
        state.cursor = 0;
        state.animation = None;
        // allow editing again
        editor_inactive.single_mut().0 = false;
    }
}

fn action_interpreter(
    input: Res<ButtonInput<KeyCode>>,
    mut tick: ResMut<AnimationTick>,
    mut state: ResMut<PlayerState>,
    mut player: Query<&mut GridTransform, With<Player>>,
    assets: Option<Res<PlayerAssets>>,
    mut next_tick: EventWriter<NextTick>,
    mut level: ResMut<Level>,
    editor_inactive: Query<&TextInputInactive, With<Editor>>,
) {
    let Ok(mut pos) = player.get_single_mut() else {
        return;
    };

    // toggle autoplay
    if input.just_pressed(KeyCode::KeyG) {
        state.autoplay = !state.autoplay;
    }

    // make sure that the editor is inactive before allowing any movement
    if !editor_inactive.single().0 {
        return;
    }

    if !tick.0.finished() {
        return;
    }
    if let Some(prev_anim) = state.animation.take() {
        pos.0 += prev_anim.final_offset(state.x_dir);
        if level.is_checkpoint(pos.0) {
            level.last_checkpoint = pos.0
        }
    }

    // check if we have manual controls to execute
    let mut animation = if cfg!(feature = "dev") {
        debug_actions(&input, &mut state).and_then(|action| {
            if let ScriptCommand::Turn = action {
                state.x_dir *= -1;
            };
            let assets = assets.as_ref().unwrap();
            level.check_valid(pos.0, action, state.x_dir, assets)
        })
    } else {
        None
    };

    // check if we have script to execute
    if input.pressed(KeyCode::KeyF) || state.autoplay {
        animation = action_interpreter_new(&mut state, &pos, &level, assets.unwrap());
    }

    // apply the animation and configure the timer
    if let Some(animation) = animation {
        tick.0.set_duration(animation.duration);
        state.animation = Some(animation);
        // next_tick.send(NextTick);
    } else {
        tick.0.set_duration(Duration::from_secs_f32(0.1));
    }
    tick.0.reset();
}

fn action_interpreter_new(
    state: &mut PlayerState,
    pos: &GridTransform,
    level: &Level,
    assets: Res<PlayerAssets>,
) -> Option<AnimationResource> {
    if state.sequence.is_empty() {
        return None;
    }

    // Rename for convenience.
    let PlayerState {
        ref sequence,
        cursor,
        ref x_dir,
        ..
    } = &mut *state;

    // Helper functions to clean up the interpreter code below.
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
                    return (i + 1) % sequence.len();
                }
                ScriptCommand::CloseBracket => count -= 1,
                _ => {}
            }
        }
        sequence.len() - 1
    };

    // Prevent infinite loops by limiting the number of iterations.
    for _ in 0..sequence.len() {
        match sequence[*cursor] {
            ScriptCommand::OpenBracket => {}
            ScriptCommand::CloseBracket => {
                // Go back to matching open bracket.
                *cursor = find_matching_open_bracket(*cursor);
            }
            command => {
                match level.check_valid(pos.0, command, *x_dir, &assets) {
                    Some(anim) => {
                        // Update the cursor.
                        *cursor = (*cursor + 1) % sequence.len();
                        // Set the animation.
                        if let ScriptCommand::Turn = command {
                            state.x_dir *= -1
                        };

                        return Some(anim.clone());
                    }
                    None => {
                        // Skip to the end of scope.
                        *cursor = find_matching_close_bracket(*cursor);
                        return None;
                    }
                }
            }
        }
        // Try the next command.
        *cursor += 1;
        *cursor %= sequence.len();
    }

    // No action from the script was possible.
    None
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
