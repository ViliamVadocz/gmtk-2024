//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use bevy::prelude::*;

use super::{
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
    Idling(usize),
    Walking(usize),
}

impl PlayerAnimationState {
    /// Return sprite index in the atlas.
    pub fn get_atlas_index(self) -> usize {
        match self {
            Self::Idling(frame) => frame,
            Self::Walking(frame) => 6 + frame,
        }
    }
}

fn propagate_grid_transform(
    mut q: Query<(
        &mut Transform,
        &GridTransform,
        &PlayerState,
        &mut TextureAtlas,
        &mut Sprite,
    )>,
    grid: Res<WorldGrid>,
    tick: Res<GridTick>,
) {
    for (mut transform, pos, state, mut atlas, mut sprite) in &mut q {
        if let Some(anim) = &state.animation {
            let frame = (anim.func)(tick.0.fraction());
            atlas.index = frame.state.get_atlas_index();
            let new = grid.project_to_world(pos.0.as_vec2() + frame.offset(state.x_dir));
            transform.translation = new.extend(transform.translation.z);
        } else {
            transform.translation = grid
                .project_to_world(pos.0.as_vec2())
                .extend(transform.translation.z);
        }
        sprite.flip_x = state.x_dir == -1;
    }
}
