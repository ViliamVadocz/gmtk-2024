//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use std::time::Duration;

use bevy::{
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
    sprite::Anchor,
};

use super::{
    action::{DOWN, RIGHT, UP},
    level::{GridTick, GridTransform, WorldGrid},
    player::PlayerState,
};
use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        propagate_grid_transform.in_set(AppSet::PropagateGridTransform),
    );
}

#[derive(Reflect, PartialEq, Clone, Copy)]
pub enum PlayerAnimationState {
    Walk(usize),
    Climb(usize),
    Drop(usize),
    Idle(usize),
}

fn propagate_grid_transform(
    mut q: Query<(
        &mut Transform,
        &GridTransform,
        &PlayerState,
        &mut TextureAtlas,
        &mut Sprite,
        &mut Handle<Image>,
    )>,
    grid: Res<WorldGrid>,
    tick: Res<GridTick>,
    player_assets: Option<Res<PlayerAssets>>,
) {
    for (mut transform, pos, state, mut atlas, mut sprite, mut texture) in &mut q {
        let anim = state
            .animation
            .as_ref()
            .unwrap_or(&player_assets.as_ref().unwrap().idle);

        let new = grid.project_to_world(pos.0.as_vec2());
        transform.translation = new.extend(transform.translation.z);

        atlas.layout = anim.atlas.clone();
        atlas.index = (tick.0.fraction() * anim.frame_count as f32) as usize;
        if state.animation.is_none() {
            atlas.index = 0;
        }

        *texture = anim.texture.clone();

        sprite.flip_x = state.x_dir == -1;
        sprite.anchor = Anchor::Custom(anim.anchor.as_vec() * Vec2::new(state.x_dir as f32, 1.));
    }
}

#[derive(Asset, Reflect, Clone)]
pub struct AnimationResource {
    #[dependency]
    pub texture: Handle<Image>,
    pub atlas: Handle<TextureAtlasLayout>,
    final_offset: IVec2,
    pub duration: Duration,
    frame_count: usize,
    anchor: Anchor,
}

impl AnimationResource {
    pub fn final_offset(&self, x_dir: i32) -> IVec2 {
        self.final_offset * IVec2::new(x_dir, 1)
    }
}

#[derive(Resource, Asset, Reflect, Clone)]
pub struct PlayerAssets {
    // This #[dependency] attribute marks the field as a dependency of the Asset.
    // This means that it will not finish loading until the labeled asset is also loaded.
    #[dependency]
    pub idle: AnimationResource,

    #[dependency]
    pub walk: AnimationResource,

    #[dependency]
    pub climb: AnimationResource,

    #[dependency]
    pub drop: AnimationResource,

    #[dependency]
    pub jump: AnimationResource,

    #[dependency]
    pub turn: AnimationResource,
}

impl PlayerAssets {
    pub const PATH_CLIMB: &'static str = "images/climb.png";
    pub const PATH_DROP: &'static str = "images/drop.png";
    pub const PATH_IDLE: &'static str = "images/idle.png";
    pub const PATH_TURN: &'static str = "images/turn.png";
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
        let drop_texture = assets.load_with_settings(PlayerAssets::PATH_DROP, settings);
        let turn_texture = assets.load_with_settings(PlayerAssets::PATH_TURN, settings);

        // A texture atlas is a way to split one image with a grid into multiple
        // sprites. By attaching it to a [`SpriteBundle`] and providing an index, we
        // can specify which section of the image we want to see. We will use this
        // to animate our player character. You can learn more about texture atlases in
        // this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let mut layout =
            |tile_size: UVec2, columns: u32, rows: u32| -> Handle<TextureAtlasLayout> {
                texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                    tile_size, columns, rows, None, None,
                ))
            };

        let idle_atlas = layout(UVec2::splat(16), 2, 2);
        let walk_atlas = layout(UVec2::new(32, 16), 4, 3);
        let climb_atlas = layout(UVec2::splat(32), 4, 3);
        let drop_atlas = layout(UVec2::splat(32), 4, 3);
        let turn_atlas = layout(UVec2::splat(16), 7, 1);

        Self {
            idle: AnimationResource {
                texture: idle_texture,
                atlas: idle_atlas,
                final_offset: IVec2::ZERO,
                duration: Duration::from_secs_f32(0.26),
                frame_count: 4,
                anchor: Anchor::Center,
            },
            walk: AnimationResource {
                texture: walk_texture,
                atlas: walk_atlas,
                final_offset: RIGHT,
                duration: Duration::from_secs_f32(0.8),
                frame_count: 12,
                anchor: Anchor::Custom(Vec2::new(-0.25, 0.)),
            },
            climb: AnimationResource {
                texture: climb_texture.clone(),
                atlas: climb_atlas.clone(),
                final_offset: UP + RIGHT,
                duration: Duration::from_secs_f32(0.73),
                frame_count: 11,
                anchor: Anchor::Custom(Vec2::new(-0.25, -0.25)),
            },
            drop: AnimationResource {
                texture: drop_texture,
                atlas: drop_atlas,
                final_offset: DOWN + RIGHT,
                duration: Duration::from_secs_f32(0.8),
                frame_count: 12,
                anchor: Anchor::Custom(Vec2::new(-0.25, 0.25)),
            },
            jump: AnimationResource {
                texture: climb_texture,
                atlas: climb_atlas,
                final_offset: RIGHT + UP + RIGHT,
                duration: Duration::from_secs_f32(0.73),
                frame_count: 11,
                anchor: Anchor::Custom(Vec2::new(-0.25, -0.25)),
            },
            turn: AnimationResource {
                texture: turn_texture,
                atlas: turn_atlas,
                final_offset: IVec2::ZERO,
                duration: Duration::from_secs_f32(0.46),
                frame_count: 7,
                anchor: Anchor::Center,
            },
        }
    }
}
