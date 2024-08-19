use bevy::prelude::*;

use super::action::ScriptCommand;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<EditorState>();
    app.add_systems(Update, (edit_script, show_script).chain());
}

#[derive(Resource, Debug, Default)]
struct EditorState {
    enabled: bool,
    entered: Vec<ScriptCommand>,
    cursor: usize,
}

#[derive(Event, Debug, Default)]
struct EditorChanged;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct EditorItem;

fn edit_script(
    input: Res<ButtonInput<KeyCode>>,
    mut editor_state: ResMut<EditorState>,
    mut editor_changed: EventWriter<EditorChanged>,
) {
    if !editor_state.enabled {
        return;
    }

    let mut changed = false;

    // Command input.
    let key_command_map = [
        (KeyCode::KeyW, ScriptCommand::Walk),
        (KeyCode::KeyC, ScriptCommand::Climb),
        (KeyCode::KeyD, ScriptCommand::Drop),
        (KeyCode::KeyI, ScriptCommand::Idle),
        (KeyCode::KeyJ, ScriptCommand::Jump),
        (KeyCode::KeyT, ScriptCommand::Turn),
        (KeyCode::BracketLeft, ScriptCommand::OpenBracket),
        (KeyCode::BracketRight, ScriptCommand::CloseBracket),
    ];
    for (key, command) in key_command_map {
        if input.just_pressed(key) {
            changed = true;
            editor_state.entered.push(command);
            editor_state.cursor += 1;
        }
    }

    // Cursor movement.
    if input.any_just_pressed([KeyCode::ArrowRight]) {
        changed = true;
        editor_state.cursor = (editor_state.cursor + 1).min(editor_state.entered.len());
    }
    if input.any_just_pressed([KeyCode::ArrowLeft]) {
        changed = true;
        editor_state.cursor = editor_state.cursor.saturating_sub(1);
    }
    if input.any_just_pressed([KeyCode::ArrowUp]) {
        changed = true;
        editor_state.cursor = 0;
    }
    if input.any_just_pressed([KeyCode::ArrowDown]) {
        changed = true;
        editor_state.cursor = editor_state.entered.len();
    }

    if changed {
        editor_changed.send_default();
    }
}

fn show_script(
    editor_state: Res<EditorState>,
    mut editor_changed: EventReader<EditorChanged>,
    query: Query<Entity, With<EditorItem>>,
) {
    for EditorChanged in editor_changed.read() {}
}
