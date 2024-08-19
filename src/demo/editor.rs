use bevy::{
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
};

use super::{action::ScriptCommand, player::PlayerState};
use crate::{asset_tracking::LoadResource, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<EditorState>();
    app.load_resource::<EditorAssets>();
    app.add_event::<EditorChanged>();
    app.add_systems(
        Update,
        ((edit_script, show_script).chain(), submit_script).run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Resource, Debug)]
pub struct EditorState {
    pub enabled: bool,
    entered: Vec<ScriptCommand>,
    cursor: usize,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            enabled: true,
            entered: Default::default(),
            cursor: Default::default(),
        }
    }
}

#[derive(Event, Debug, Default)]
struct EditorChanged;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct EditorItem;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct EditorUI;

#[derive(Resource, Asset, Reflect, Clone)]
pub struct EditorAssets {
    // This #[dependency] attribute marks the field as a dependency of the Asset.
    // This means that it will not finish loading until the labeled asset is also loaded.
    #[dependency]
    pub icons: Handle<Image>,
    pub atlas: Handle<TextureAtlasLayout>,

    #[dependency]
    pub cursor: Handle<Image>,
}

impl EditorAssets {
    pub const PATH_CURSOR: &'static str = "images/cursor.png";
    pub const PATH_ICONS: &'static str = "images/icons.png";

    fn get_atlas_index(command: &ScriptCommand) -> usize {
        match command {
            ScriptCommand::Walk => 0,
            ScriptCommand::Climb => 1,
            ScriptCommand::Drop => 2,
            ScriptCommand::Idle => 3,
            ScriptCommand::Turn => 4,
            ScriptCommand::Jump => 5,
            ScriptCommand::OpenBracket => 6,
            ScriptCommand::CloseBracket => 7,
        }
    }
}

impl FromWorld for EditorAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        let icons = assets.load_with_settings(
            EditorAssets::PATH_ICONS,
            |settings: &mut ImageLoaderSettings| {
                // Use `nearest` image sampling to preserve the pixel art style.
                settings.sampler = ImageSampler::nearest();
            },
        );

        let cursor = assets.load_with_settings(
            EditorAssets::PATH_CURSOR,
            |settings: &mut ImageLoaderSettings| {
                // Use `nearest` image sampling to preserve the pixel art style.
                settings.sampler = ImageSampler::nearest();
            },
        );

        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let atlas = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(16),
            1,
            8,
            None,
            None,
        ));

        Self {
            icons,
            atlas,
            cursor,
        }
    }
}

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
            let index = editor_state.cursor;
            editor_state.entered.insert(index, command);
            editor_state.cursor += 1;
        }
    }

    // Cursor movement.
    if input.just_pressed(KeyCode::ArrowRight) {
        changed = true;
        editor_state.cursor = (editor_state.cursor + 1).min(editor_state.entered.len());
    }
    if input.just_pressed(KeyCode::ArrowLeft) {
        changed = true;
        editor_state.cursor = editor_state.cursor.saturating_sub(1);
    }
    if input.just_pressed(KeyCode::ArrowUp) {
        changed = true;
        editor_state.cursor = 0;
    }
    if input.just_pressed(KeyCode::ArrowDown) {
        changed = true;
        editor_state.cursor = editor_state.entered.len();
    }

    // Delete stuff.
    if input.just_pressed(KeyCode::Backspace) {
        changed = true;
        let index = editor_state.cursor;
        if index > 0 {
            editor_state.entered.remove(index - 1);
        }
        editor_state.cursor = editor_state.cursor.saturating_sub(1);
    }
    if input.just_pressed(KeyCode::Delete) {
        changed = true;
        let index = editor_state.cursor;
        if index < editor_state.entered.len() {
            editor_state.entered.remove(index);
        }
    }

    if changed {
        editor_changed.send_default();
    }
}

fn show_script(
    editor_state: Res<EditorState>,
    mut editor_changed: EventReader<EditorChanged>,
    mut commands: Commands,
    editor_ui_query: Query<Entity, With<EditorUI>>,
    editor_items_query: Query<Entity, (With<EditorItem>, Without<EditorUI>)>,
    editor_assets: Res<EditorAssets>,
) {
    let Some(EditorChanged) = editor_changed.read().next() else {
        return;
    };

    // TODO: Actually use this to fix the sequence (visually).
    let bracket_balance = calculate_bracket_balance(&editor_state.entered);

    // Despawn all current editor item entities.
    for entity in &editor_items_query {
        commands.entity(entity).despawn_recursive();
    }

    log::info!("{editor_state:?}");

    // Spawn new editor items.
    let editor_ui = editor_ui_query.single();
    commands.entity(editor_ui).with_children(|children| {
        for _ in bracket_balance..0 {
            spawn_editor_item(
                &editor_assets,
                children,
                &ScriptCommand::OpenBracket,
                Color::linear_rgba(0.0, 0.0, 0.0, 0.5),
            );
        }
        for command in &editor_state.entered {
            spawn_editor_item(
                &editor_assets,
                children,
                command,
                Color::linear_rgba(0.0, 0.0, 0.0, 1.0),
            );
        }
        for _ in 0..bracket_balance {
            spawn_editor_item(
                &editor_assets,
                children,
                &ScriptCommand::CloseBracket,
                Color::linear_rgba(0.0, 0.0, 0.0, 0.5),
            );
        }
    });
}

fn spawn_editor_item(
    editor_assets: &EditorAssets,
    children: &mut ChildBuilder,
    command: &ScriptCommand,
    color: Color,
) {
    children.spawn((
        ImageBundle {
            style: Style {
                height: Val::Percent(100.0),
                ..default()
            },
            image: UiImage::new(editor_assets.icons.clone()).with_color(color),
            ..default()
        },
        TextureAtlas {
            layout: editor_assets.atlas.clone(),
            index: EditorAssets::get_atlas_index(command),
        },
        EditorItem,
    ));
}

fn calculate_bracket_balance(script: &[ScriptCommand]) -> i32 {
    let mut balance = 0;
    for command in script {
        match command {
            ScriptCommand::OpenBracket => balance += 1,
            ScriptCommand::CloseBracket => balance -= 1,
            _ => {}
        }
    }
    balance
}

fn submit_script(
    input: Res<ButtonInput<KeyCode>>,
    mut editor_state: ResMut<EditorState>,
    mut player_state: ResMut<PlayerState>,
    mut editor_changed: EventWriter<EditorChanged>,
) {
    if !input.just_pressed(KeyCode::Enter) {
        return;
    }

    if editor_state.entered.is_empty() {
        return;
    }

    // Fix sequence (brackets-wise)
    let bracket_balance = calculate_bracket_balance(&editor_state.entered);
    let new_sequence: Vec<_> = (bracket_balance..0)
        .map(|_| ScriptCommand::OpenBracket)
        .chain(editor_state.entered.drain(..))
        .chain((0..bracket_balance).map(|_| ScriptCommand::CloseBracket))
        .collect();
    editor_state.entered = new_sequence.clone();
    editor_state.cursor = new_sequence.len();
    // Send event to update the editor view.
    editor_changed.send_default();

    // TODO: sequence checks (unlock-based stuff)

    editor_state.enabled = false;
    player_state.sequence = new_sequence;
    player_state.cursor = 0;
}
