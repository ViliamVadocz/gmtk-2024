//! Plugin handling the player character in particular.
//! Note that this is separate from the `movement` module as that could be used
//! for other characters as well.

use std::time::Duration;

use bevy::{
    ecs::{system::RunSystemOnce as _, world::Command},
    prelude::*,
};

use super::{
    action::ScriptCommand,
    animation::{AnimationResource, PlayerAssets},
    editor::EditorState,
    level::{AnimationTick, GridTransform, Level},
};
use crate::{
    asset_tracking::LoadResource,
    demo::{
        editor::{EditorAssets, ShowEditor},
        level::{NextGridTransform, Reset, TickStart},
        obstacle::Obstacle,
    },
    screens::gameplay::{AutoplayLabel, UnlockedList},
    theme::palette::LABEL_TEXT,
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
            update_animation.in_set(AppSet::RecordInput),
            camera_follow_player.in_set(AppSet::UpdateCamera),
        ),
    );
    app.insert_resource(PlayerState {
        x_dir: 1,
        animation: None,
        sequence: vec![],
        cursor: 0,
        autoplay: true,
    });
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Resource)]
pub struct PlayerState {
    // can be 1 or -1
    pub x_dir: i32,
    pub animation: Option<AnimationResource>,

    pub sequence: Vec<ScriptCommand>,
    pub cursor: usize,
    pub autoplay: bool,
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

pub struct AddUnlockedCommand {
    pub command: ScriptCommand,
}

impl Command for AddUnlockedCommand {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self, add_unlock)
    }
}

fn add_unlock(
    config: In<AddUnlockedCommand>,
    unlocked_list: Query<Entity, With<UnlockedList>>,
    editor_assets: Res<EditorAssets>,
    mut commands: Commands,
) {
    commands
        .entity(unlocked_list.single())
        .with_children(|children| {
            children.spawn((
                ImageBundle {
                    style: Style {
                        height: Val::Percent(10.0),
                        ..default()
                    },
                    image: UiImage::new(editor_assets.icons.clone()),
                    ..default()
                },
                TextureAtlas {
                    layout: editor_assets.atlas.clone(),
                    index: EditorAssets::get_atlas_index(&config.command),
                },
            ));
        });
}

fn respawn(
    mut state: ResMut<PlayerState>,
    mut player: Query<(&mut GridTransform, &mut NextGridTransform), With<Player>>,
    obstacles: Query<&GridTransform, (With<Obstacle>, Without<Player>)>,
    input: Res<ButtonInput<KeyCode>>,
    mut level: ResMut<Level>,
    mut reset: EventWriter<Reset>,
    mut editor_state: ResMut<EditorState>,
    mut commands: Commands,
) {
    let Ok((mut pos, mut new_pos)) = player.get_single_mut() else {
        return;
    };

    let mut collided = false;
    for o_pos in &obstacles {
        collided |= o_pos.0 == pos.0;
    }

    if level.is_checkpoint(pos.0) && level.last_checkpoint != pos.0 {
        level.last_checkpoint = pos.0;

        let (new_unlock, command_count) = *level.unlocks.get(&pos.0).expect("unknown checkpoint");
        if let Some(script_command) = new_unlock {
            if !level.unlocked.contains(&script_command) {
                level.unlocked.push(script_command);
                commands.add(AddUnlockedCommand {
                    command: script_command,
                });
            }
        }
        level.command_count = level.command_count.max(command_count);

        collided = true;
    }

    if input.just_pressed(KeyCode::KeyR) || collided {
        // respawn, reset all properties
        pos.0 = level.last_checkpoint;
        new_pos.0 = level.last_checkpoint;
        state.x_dir = 1;
        state.cursor = 0;
        state.animation = None;
        // allow editing again
        editor_state.enabled = true;
        reset.send(Reset);
        commands.add(ShowEditor::default());
    }
}

fn update_animation(
    input: Res<ButtonInput<KeyCode>>,
    mut tick: ResMut<AnimationTick>,
    mut state: ResMut<PlayerState>,
    mut player: Query<(&GridTransform, &mut NextGridTransform), With<Player>>,
    assets: Option<Res<PlayerAssets>>,
    level: Res<Level>,
    editor_state: Res<EditorState>,
    mut tick_start: EventWriter<TickStart>,
    mut autoplay_label: Query<&mut Text, With<AutoplayLabel>>,
    mut commands: Commands,
) {
    let Ok((pos, mut next_pos)) = player.get_single_mut() else {
        return;
    };

    // toggle autoplay
    if input.just_pressed(KeyCode::KeyG) {
        let mut autoplay_label = autoplay_label.single_mut();

        state.autoplay = !state.autoplay;
        let default_style = TextStyle {
            font_size: 24.0,
            color: LABEL_TEXT,
            ..Default::default()
        };
        let big_style = TextStyle {
            font_size: 48.0,
            color: LABEL_TEXT,
            ..Default::default()
        };
        *autoplay_label = match state.autoplay {
            true => Text::from_section(AutoplayLabel::ENABLED, default_style).with_no_wrap(),
            false => Text::from_sections([
                TextSection::new(AutoplayLabel::DISABLED_BIG, big_style),
                TextSection::new(AutoplayLabel::DISABLED, default_style),
            ])
            .with_no_wrap(),
        };
    }

    // make sure that the editor is disabled before allowing any movement
    if editor_state.enabled {
        return;
    }

    if !tick.0.finished() {
        return;
    }

    state.animation = None;

    // check if we have manual controls to execute
    if cfg!(feature = "dev") {
        state.animation = debug_actions(&input, &mut state).and_then(|action| {
            if let ScriptCommand::Turn = action {
                state.x_dir *= -1;
            };
            let assets = assets.as_ref().unwrap();
            level.check_valid(pos.0, action, state.x_dir, assets)
        })
    };

    // check if we have script to execute
    if input.pressed(KeyCode::KeyF) || state.autoplay {
        let (script_index, animation) =
            action_interpreter(&mut state, pos, &level, assets.unwrap());
        state.animation = animation;
        commands.add(ShowEditor {
            active: Some((script_index, state.animation.is_some())),
        });
    }

    let multiplier = if state.autoplay && input.pressed(KeyCode::KeyF) {
        0.25
    } else {
        1.0
    };

    if let Some(animation) = &state.animation {
        tick_start.send(TickStart);
        tick.0.set_duration(animation.duration.mul_f32(multiplier));
        next_pos.0 = pos.0 + animation.final_offset(state.x_dir);
        tick.0.reset();
    } else if state.autoplay {
        tick.0
            .set_duration(Duration::from_secs_f32(0.25).mul_f32(multiplier));
        tick.0.reset();
    }
}

/// Returns the index of the script item that should be highlighted and maybe
/// the animation that should be played.
fn action_interpreter(
    state: &mut PlayerState,
    pos: &GridTransform,
    level: &Level,
    assets: Res<PlayerAssets>,
) -> (usize, Option<AnimationResource>) {
    if state.sequence.is_empty() {
        log::error!("The sequence should never be empty!");
        return (0, None);
    }

    // Rename for convenience.
    let state = &mut *state;
    let cursor = &mut state.cursor;
    let sequence = &state.sequence;

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
        0
    };

    // Prevent infinite loops by limiting the number of iterations.
    for _ in 0..sequence.len() {
        let item_index = *cursor;
        match sequence[*cursor] {
            ScriptCommand::OpenBracket => {}
            ScriptCommand::CloseBracket => {
                // Go back to matching open bracket.
                *cursor = find_matching_open_bracket(*cursor);
            }
            command => {
                match level.check_valid(pos.0, command, state.x_dir, &assets) {
                    Some(anim) => {
                        // Update the cursor.
                        *cursor = (*cursor + 1) % sequence.len();
                        // Set the animation.
                        if let ScriptCommand::Turn = command {
                            state.x_dir *= -1
                        };

                        return (item_index, Some(anim.clone()));
                    }
                    None => {
                        // Skip to the end of scope.
                        *cursor = find_matching_close_bracket(*cursor);
                        return (item_index, None);
                    }
                }
            }
        }
        // Try the next command.
        *cursor += 1;
        *cursor %= sequence.len();
    }

    // No action from the script was possible.
    (*cursor, None)
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
