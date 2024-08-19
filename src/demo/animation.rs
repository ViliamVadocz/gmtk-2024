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
    level::{AnimationTick, GridTransform, WorldGrid},
    player::PlayerState,
};
use crate::{demo::player::Player, AppSet};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, apply_animation.in_set(AppSet::ApplyAnimation));
}

#[derive(Reflect, PartialEq, Clone, Copy)]
pub enum PlayerAnimationState {
    Walk(usize),
    Climb(usize),
    Drop(usize),
    Idle(usize),
}

fn apply_animation(
    state: Res<PlayerState>,
    mut q: Query<
        (
            &mut Transform,
            &GridTransform,
            &mut TextureAtlas,
            &mut Sprite,
        ),
        With<Player>,
    >,
    grid: Res<WorldGrid>,
    tick: Res<AnimationTick>,
    player_assets: Option<Res<PlayerAssets>>,
) {
    let Ok((mut transform, pos, mut atlas, mut sprite)) = q.get_single_mut() else {
        return;
    };

    let anim = state
        .animation
        .as_ref()
        .unwrap_or(&player_assets.as_ref().unwrap().idle);

    let new = grid.project_to_world(pos.0.as_vec2());
    transform.translation = new.extend(transform.translation.z);

    atlas.index = anim.row_number * (PlayerAssets::ANIM_COLUMNS as usize)
        + (tick.0.fraction() * anim.frame_count as f32) as usize;
    if state.animation.is_none() {
        atlas.index = 0;
    }

    sprite.flip_x = state.x_dir == -1;
    sprite.anchor = Anchor::Custom(anim.anchor.as_vec() * Vec2::new(state.x_dir as f32, 1.));
}

#[derive(Clone, Reflect)]
pub struct AnimationResource {
    pub squares: Vec<IVec2>,
    pub duration: Duration,
    frame_count: usize,
    anchor: Anchor,
    row_number: usize,
}

impl AnimationResource {
    pub fn final_offset(&self, x_dir: i32) -> IVec2 {
        self.squares.last().copied().unwrap_or(IVec2::ZERO) * IVec2::new(x_dir, 1)
    }
}

#[derive(Resource, Asset, Reflect, Clone)]
pub struct PlayerAssets {
    // This #[dependency] attribute marks the field as a dependency of the Asset.
    // This means that it will not finish loading until the labeled asset is also loaded.
    pub idle: AnimationResource,

    pub walk: AnimationResource,

    pub climb: AnimationResource,

    pub drop: AnimationResource,

    pub drop2: AnimationResource,

    pub jump: AnimationResource,

    pub turn: AnimationResource,

    #[dependency]
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,

    #[dependency]
    pub hazard_texture: Handle<Image>,
    pub hazard_layout: Handle<TextureAtlasLayout>,
}

impl PlayerAssets {
    pub const ANIM_COLUMNS: u32 = 16;
    pub const ANIM_ROWS: u32 = 8;
    pub const HAZARD_PATH: &'static str = "images/hazard.png";
    pub const PATH: &'static str = "images/robot.png";
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        let settings = |settings: &mut ImageLoaderSettings| {
            // Use `nearest` image sampling to preserve the pixel art style.
            settings.sampler = ImageSampler::nearest();
        };

        let texture = assets.load_with_settings(PlayerAssets::PATH, settings);
        let hazard_texture = assets.load_with_settings(PlayerAssets::HAZARD_PATH, settings);

        // A texture atlas is a way to split one image with a grid into multiple
        // sprites. By attaching it to a [`SpriteBundle`] and providing an index, we
        // can specify which section of the image we want to see. We will use this
        // to animate our player character. You can learn more about texture atlases in
        // this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(48),
            PlayerAssets::ANIM_COLUMNS,
            PlayerAssets::ANIM_ROWS,
            None,
            None,
        ));
        let hazard_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(16),
            4,
            1,
            None,
            None,
        ));

        Self {
            idle: AnimationResource {
                squares: vec![],
                duration: Duration::from_secs_f32(0.8),
                frame_count: 4,
                anchor: Anchor::Center,
                row_number: 0,
            },
            walk: AnimationResource {
                squares: vec![RIGHT],
                duration: Duration::from_secs_f32(0.8),
                frame_count: 12,
                anchor: Anchor::Center,
                row_number: 1,
            },
            climb: AnimationResource {
                squares: vec![UP, UP + RIGHT],
                duration: Duration::from_secs_f32(0.8),
                frame_count: 10,
                anchor: Anchor::Center,
                row_number: 2,
            },
            turn: AnimationResource {
                squares: vec![],
                duration: Duration::from_secs_f32(0.8),
                frame_count: 7,
                anchor: Anchor::Center,
                row_number: 3,
            },
            drop: AnimationResource {
                squares: vec![RIGHT, DOWN + RIGHT],
                duration: Duration::from_secs_f32(0.8),
                frame_count: 11,
                anchor: Anchor::Center,
                row_number: 4,
            },
            drop2: AnimationResource {
                squares: vec![RIGHT, DOWN + RIGHT, DOWN + DOWN + RIGHT],
                duration: Duration::from_secs_f32(0.8),
                frame_count: 12,
                anchor: Anchor::Custom(Vec2::new(0.0, 1.0 / 3.0)),
                row_number: 5,
            },
            jump: AnimationResource {
                squares: vec![RIGHT, UP, RIGHT + UP, RIGHT + UP + RIGHT],
                duration: Duration::from_secs_f32(0.8),
                frame_count: 13,
                anchor: Anchor::Custom(Vec2::new(-1.0 / 3.0, 0.0)),
                row_number: 6,
            },
            texture,
            layout,
            hazard_layout,
            hazard_texture,
        }
    }
}
