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
    level::{GridTransform, WorldGrid},
};
use crate::{screens::Screen, AppSet};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(AnimationTick(Timer::from_seconds(1.0, TimerMode::Once)));
    app.add_event::<TickEvent>();
    app.add_systems(
        Update,
        (
            update_tick_timer.in_set(AppSet::TickTimers),
            (update_grid_position, update_animation)
                .chain()
                .in_set(AppSet::UpdateAnimationAndTransform),
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Reflect, PartialEq, Clone, Copy)]
pub enum PlayerAnimationState {
    Walk(usize),
    Climb(usize),
    Drop(usize),
    Idle(usize),
}

fn update_grid_position(
    mut query: Query<(&AnimationState, &mut GridTransform)>,
    mut event_reader: EventReader<TickEvent>,
) {
    for TickEvent in event_reader.read() {
        for (state, mut pos) in &mut query {
            if let Some(anim) = &state.animation {
                pos.0 += anim.final_offset(state.x_dir);
            }
        }
    }
}

fn update_animation(
    mut q: Query<(
        &mut Transform,
        &GridTransform,
        &AnimationState,
        &mut TextureAtlas,
        &mut Sprite,
        &mut Handle<Image>,
    )>,
    grid: Res<WorldGrid>,
    tick: Res<AnimationTick>,
    player_assets: Option<Res<PlayerAssets>>,
) {
    for (mut transform, pos, state, mut atlas, mut sprite, mut texture) in &mut q {
        // Get the current animation resource or default to Idle.
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
    pub squares: Vec<IVec2>,
    pub duration: Duration,
    frame_count: usize,
    anchor: Anchor,
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
                squares: vec![],
                duration: Duration::from_secs_f32(0.26),
                frame_count: 4,
                anchor: Anchor::Center,
            },
            walk: AnimationResource {
                texture: walk_texture,
                atlas: walk_atlas,
                squares: vec![RIGHT],
                duration: Duration::from_secs_f32(0.8),
                frame_count: 12,
                anchor: Anchor::Custom(Vec2::new(-0.25, 0.)),
            },
            climb: AnimationResource {
                texture: climb_texture.clone(),
                atlas: climb_atlas.clone(),
                squares: vec![UP, UP + RIGHT],
                duration: Duration::from_secs_f32(0.73),
                frame_count: 11,
                anchor: Anchor::Custom(Vec2::new(-0.25, -0.25)),
            },
            drop: AnimationResource {
                texture: drop_texture,
                atlas: drop_atlas,
                squares: vec![RIGHT, DOWN + RIGHT],
                duration: Duration::from_secs_f32(0.8),
                frame_count: 12,
                anchor: Anchor::Custom(Vec2::new(-0.25, 0.25)),
            },
            jump: AnimationResource {
                texture: climb_texture,
                atlas: climb_atlas,
                squares: vec![RIGHT, UP, RIGHT + UP, RIGHT + UP + RIGHT],
                duration: Duration::from_secs_f32(0.73),
                frame_count: 11,
                anchor: Anchor::Custom(Vec2::new(-0.25, -0.25)),
            },
            turn: AnimationResource {
                texture: turn_texture,
                atlas: turn_atlas,
                squares: vec![],
                duration: Duration::from_secs_f32(0.46),
                frame_count: 7,
                anchor: Anchor::Center,
            },
        }
    }
}

#[derive(Resource)]
pub struct AnimationTick(pub Timer);

pub fn update_tick_timer(
    time: Res<Time>,
    mut tick: ResMut<AnimationTick>,
    mut event_writer: EventWriter<TickEvent>,
) {
    tick.0.tick(time.delta());
    if tick.0.just_finished() {
        event_writer.send_default();
    }
}

#[derive(Event, Default)]
pub struct TickEvent;

#[derive(Component)]
pub struct AnimationState {
    // can be 1 or -1
    pub x_dir: i32,
    pub animation: Option<AnimationResource>,
}
