//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {}

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
