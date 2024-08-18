//! Plugin handling the player character in particular.
//! Note that this is separate from the `movement` module as that could be used
//! for other characters as well.

use bevy::{
    ecs::{system::RunSystemOnce as _, world::Command},
    prelude::*,
    sprite::Anchor,
};

use super::{
    action::PlayerAction,
    animation::{AnimationResource, PlayerAssets},
    level::{GridTick, GridTransform, Level, NextTick},
};
use crate::{asset_tracking::LoadResource, screens::Screen, AppSet};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            record_player_directional_input.in_set(AppSet::RecordInput),
            camera_follow_player.in_set(AppSet::UpdateCamera),
        ),
    );
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

#[derive(Component)]
pub struct PlayerState {
    // can be 1 or -1
    pub x_dir: i32,
    pub animation: Option<AnimationResource>,
    pub sequence: Vec<PlayerAction>,
    pub cursor: usize,
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
        PlayerState {
            x_dir: 1,
            animation: None,
            sequence: vec![PlayerAction::Walk, PlayerAction::Climb],
            cursor: 0,
        },
        TextureAtlas {
            layout: player_assets.idle.atlas.clone(),
            index: 0,
        },
        StateScoped(Screen::Gameplay),
    ));
}

fn which_action(input: &ButtonInput<KeyCode>, state: &mut PlayerState) -> Option<PlayerAction> {
    if input.pressed(KeyCode::KeyF) {
        let action = state.sequence[state.cursor];
        state.cursor = (state.cursor + 1) % state.sequence.len();
        return Some(action);
    }

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
            return Some(PlayerAction::Turn);
        }
        action = Some(PlayerAction::Walk)
    }
    if pressed_or_held(KeyCode::KeyW) || pressed_or_held(KeyCode::ArrowUp) {
        action = Some(PlayerAction::Climb)
    }
    if pressed_or_held(KeyCode::KeyS) || pressed_or_held(KeyCode::ArrowDown) {
        action = Some(PlayerAction::Drop)
    }
    if pressed_or_held(KeyCode::Space) {
        action = Some(PlayerAction::Idle)
    }
    return action;
}

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut tick: ResMut<GridTick>,
    mut player: Query<(&mut GridTransform, &mut PlayerState), With<Player>>,
    assets: Option<Res<PlayerAssets>>,
    mut next_tick: EventWriter<NextTick>,
    level: Res<Level>,
) {
    let Ok((mut pos, mut state)) = player.get_single_mut() else {
        return;
    };

    if !tick.0.finished() {
        return;
    }
    if let Some(prev_anim) = state.animation.take() {
        pos.0 += prev_anim.final_offset(state.x_dir);
    }

    if let Some(action) = which_action(&input, &mut state) {
        let assets = assets.as_ref().unwrap();
        let Some(animation) = level.check_valid(pos.0, action, state.x_dir, assets) else {
            return;
        };

        if let PlayerAction::Turn = action {
            state.x_dir *= -1;
        }

        tick.0.reset();
        tick.0.set_duration(animation.duration);
        state.animation = Some(animation);
        next_tick.send(NextTick);
    }
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
