use std::time::Duration;

use bevy::prelude::*;

use super::{animation::PlayerAnimationState, level::Level};
pub enum PlayerAction {
    Walk,
    Climb,
}

const UP: IVec2 = IVec2::new(0, 1);
const DOWN: IVec2 = IVec2::new(0, -1);
const LEFT: IVec2 = IVec2::new(-1, 0);
const RIGHT: IVec2 = IVec2::new(1, 0);

impl Level {
    pub fn check_valid(&self, pos: IVec2, action: PlayerAction, x_dir: i32) -> Option<Animation> {
        action
            .squares()
            .all(|square| !self.is_solid(pos + square * IVec2::new(x_dir, 1)))
            .then(|| action.animation())
    }
}

pub struct Animation {
    pub destination: IVec2,
    pub duration: Duration,
    // returns
    pub func: Box<dyn Fn(f32) -> (Vec2, PlayerAnimationState) + Send + Sync>,
}

impl PlayerAction {
    // the squares in the order that they are touched
    pub fn squares(&self) -> impl Iterator<Item = IVec2> {
        match self {
            PlayerAction::Walk => vec![RIGHT],
            PlayerAction::Climb => vec![UP, UP + RIGHT],
        }
        .into_iter()
    }

    // animations
    pub fn animation(&self) -> Animation {
        match self {
            PlayerAction::Walk => Animation {
                destination: RIGHT,
                duration: Duration::from_secs_f32(0.2),
                func: Box::new(|f| {
                    (
                        RIGHT.as_vec2() * f,
                        PlayerAnimationState::Walking((f * 6.) as usize),
                    )
                }),
            },
            PlayerAction::Climb => Animation {
                destination: UP + RIGHT,
                duration: Duration::from_secs_f32(0.5),
                func: Box::new(|f| {
                    (
                        RIGHT.as_vec2() * f,
                        PlayerAnimationState::Walking((f * 6.) as usize),
                    )
                }),
            },
        }
    }
}
