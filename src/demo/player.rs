//! Plugin handling the player character in particular.
//! Note that this is separate from the `movement` module as that could be used
//! for other characters as well.

use bevy::{
    ecs::{system::RunSystemOnce as _, world::Command},
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
    sprite::Anchor,
};

use super::{
    action::{Animation, PlayerAction},
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
    pub animation: Option<Animation>,
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
            texture: player_assets.walk_texture.clone(),
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
        },
        TextureAtlas {
            layout: player_assets.walk_atlas.clone(),
            index: 0,
        },
        StateScoped(Screen::Gameplay),
    ));
}

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut tick: ResMut<GridTick>,
    mut player: Query<(&mut GridTransform, &mut PlayerState), With<Player>>,
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

    let pressed_or_held =
        |key: KeyCode| tick.0.finished() && input.pressed(key) || input.just_pressed(key);

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
        state.x_dir = facing;
        action = Some(PlayerAction::Walk)
    }
    if pressed_or_held(KeyCode::KeyW) || pressed_or_held(KeyCode::ArrowUp) {
        action = Some(PlayerAction::Climb)
    }
    if let Some(action) = action {
        let Some(animation) = level.check_valid(pos.0, action, state.x_dir) else {
            return;
        };

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

#[derive(Resource, Asset, Reflect, Clone)]
pub struct PlayerAssets {
    // This #[dependency] attribute marks the field as a dependency of the Asset.
    // This means that it will not finish loading until the labeled asset is also loaded.
    #[dependency]
    pub idle_texture: Handle<Image>,
    // #[dependency]
    pub idle_atlas: Handle<TextureAtlasLayout>,

    #[dependency]
    pub climb_texture: Handle<Image>,
    // #[dependency]
    pub climb_atlas: Handle<TextureAtlasLayout>,

    #[dependency]
    pub walk_texture: Handle<Image>,
    // #[dependency]
    pub walk_atlas: Handle<TextureAtlasLayout>,
}

impl PlayerAssets {
    pub const PATH_CLIMB: &'static str = "images/climb.png";
    pub const PATH_IDLE: &'static str = "images/idle.png";
    pub const PATH_WALK: &'static str = "images/walk.png";
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        let settings = |settings: &mut ImageLoaderSettings| {
            // Use `nearest` image sampling to preserve the pixel art style.
            settings.sampler = ImageSampler::nearest();
        };

        let idle_texture = assets.load_with_settings(PlayerAssets::PATH_IDLE, settings);
        let walk_texture = assets.load_with_settings(PlayerAssets::PATH_WALK, settings);
        let climb_texture = assets.load_with_settings(PlayerAssets::PATH_CLIMB, settings);

        // A texture atlas is a way to split one image with a grid into multiple
        // sprites. By attaching it to a [`SpriteBundle`] and providing an index, we
        // can specify which section of the image we want to see. We will use this
        // to animate our player character. You can learn more about texture atlases in
        // this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let idle_atlas = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(16),
            2,
            2,
            None,
            None,
        ));
        let walk_atlas = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(32, 16),
            4,
            3,
            None,
            None,
        ));
        let climb_atlas = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            3,
            4,
            None,
            None,
        ));

        Self {
            idle_texture,
            walk_texture,
            climb_texture,
            idle_atlas,
            walk_atlas,
            climb_atlas,
        }
    }
}
