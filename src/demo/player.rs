//! Plugin handling the player character in particular.
//! Note that this is separate from the `movement` module as that could be used
//! for other characters as well.

use bevy::{
    ecs::{system::RunSystemOnce as _, world::Command},
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
};

use super::level::{GridTick, GridTransform, NextTick, OldGridTransform};
use crate::{
    asset_tracking::LoadResource,
    demo::animation::PlayerAnimation,
    screens::Screen,
    AppSet,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        record_player_directional_input.in_set(AppSet::RecordInput),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

/// A command to spawn the player character.
#[derive(Debug)]
pub struct SpawnPlayer {
    /// See [`MovementController::max_speed`].
    pub max_speed: f32,
}

impl Command for SpawnPlayer {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self, spawn_player);
    }
}

fn spawn_player(
    In(config): In<SpawnPlayer>,
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // A texture atlas is a way to split one image with a grid into multiple
    // sprites. By attaching it to a [`SpriteBundle`] and providing an index, we
    // can specify which section of the image we want to see. We will use this
    // to animate our player character. You can learn more about texture atlases in
    // this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    commands.spawn((
        Name::new("Player"),
        Player,
        SpriteBundle {
            texture: player_assets.ducky.clone(),
            transform: Transform::from_scale(Vec2::splat(2.0).extend(1.0)),
            ..Default::default()
        },
        GridTransform(IVec2::default()),
        OldGridTransform(vec![]),
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: player_animation.get_atlas_index(),
        },
        player_animation,
        StateScoped(Screen::Gameplay),
    ));
}

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut tick: ResMut<GridTick>,
    mut pos: Query<(&mut GridTransform, &mut OldGridTransform), With<Player>>,
    mut next_tick: EventWriter<NextTick>,
) {
    if input.just_pressed(KeyCode::Backspace) {
        for (mut new, mut old) in &mut pos {
            if let Some(old) = old.0.pop() {
                new.0 = old;
            }
        }

        let end = tick.0.duration();
        tick.0.set_elapsed(end);
        return;
    }

    let pressed_or_held =
        |key: KeyCode| tick.0.finished() && input.pressed(key) || input.just_pressed(key);

    // Collect directional input.
    let mut intent = IVec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1;
    }
    if pressed_or_held(KeyCode::KeyA) || pressed_or_held(KeyCode::ArrowLeft) {
        intent.x -= 1;
    }
    if pressed_or_held(KeyCode::KeyD) || pressed_or_held(KeyCode::ArrowRight) {
        intent.x += 1;
    }
    if intent.x != 0 {
        for (mut new, mut old) in &mut pos {
            old.0.push(new.0);
            new.0 += intent;
        }

        tick.0.reset();
        next_tick.send(NextTick);
    }
}

#[derive(Resource, Asset, Reflect, Clone)]
pub struct PlayerAssets {
    // This #[dependency] attribute marks the field as a dependency of the Asset.
    // This means that it will not finish loading until the labeled asset is also loaded.
    #[dependency]
    pub ducky: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl PlayerAssets {
    pub const PATH_DUCKY: &'static str = "images/ducky.png";
    pub const PATH_STEP_1: &'static str = "audio/sound_effects/step1.ogg";
    pub const PATH_STEP_2: &'static str = "audio/sound_effects/step2.ogg";
    pub const PATH_STEP_3: &'static str = "audio/sound_effects/step3.ogg";
    pub const PATH_STEP_4: &'static str = "audio/sound_effects/step4.ogg";
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load_with_settings(
                PlayerAssets::PATH_DUCKY,
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve the pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            steps: vec![
                assets.load(PlayerAssets::PATH_STEP_1),
                assets.load(PlayerAssets::PATH_STEP_2),
                assets.load(PlayerAssets::PATH_STEP_3),
                assets.load(PlayerAssets::PATH_STEP_4),
            ],
        }
    }
}
