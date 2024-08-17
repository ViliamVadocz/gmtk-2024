//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use bevy::{prelude::*, sprite::Anchor};

use super::{
    level::{GridTick, GridTransform, WorldGrid},
    player::{PlayerAssets, PlayerState},
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
    Idle(usize),
    Walk(usize),
    Climb(usize),
}

impl PlayerAnimationState {
    pub fn update_animation(
        self,
        player_assets: &PlayerAssets,
        atlas: &mut TextureAtlas,
        texture: &mut Handle<Image>,
        anchor: &mut Anchor,
    ) {
        match self {
            Self::Idle(i) => {
                atlas.layout = player_assets.idle_atlas.clone();
                atlas.index = i;
                *texture = player_assets.idle_texture.clone();
                *anchor = Anchor::Center;
            }
            Self::Walk(i) => {
                atlas.layout = player_assets.walk_atlas.clone();
                atlas.index = i;
                *texture = player_assets.walk_texture.clone();
                *anchor = Anchor::Custom(Vec2::new(-0.25, 0.));
            }
            Self::Climb(i) => {
                atlas.layout = player_assets.climb_atlas.clone();
                atlas.index = i;
                *texture = player_assets.climb_texture.clone();
                *anchor = Anchor::Custom(Vec2::new(-0.25, -0.25));
            }
        };
    }
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
        let anim_state = if let Some(anim) = &state.animation {
            let frame = (anim.func)(tick.0.fraction());

            let new = grid.project_to_world(pos.0.as_vec2() + frame.offset(state.x_dir));
            transform.translation = new.extend(transform.translation.z);
            frame.state
        } else {
            transform.translation = grid
                .project_to_world(pos.0.as_vec2())
                .extend(transform.translation.z);
            PlayerAnimationState::Idle(0)
        };
        anim_state.update_animation(
            player_assets
                .as_ref()
                .expect("Player assets should already be loaded."),
            &mut atlas,
            &mut texture,
            &mut sprite.anchor,
        );
        sprite.flip_x = state.x_dir == -1;
        sprite.anchor = Anchor::Custom(sprite.anchor.as_vec() * Vec2::new(state.x_dir as f32, 1.));
    }
}
