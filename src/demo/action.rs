use std::time::Duration;

use bevy::prelude::*;

use super::{animation::PlayerAnimationState, level::Level};

const UP: IVec2 = IVec2::new(0, 1);
const DOWN: IVec2 = IVec2::new(0, -1);
const LEFT: IVec2 = IVec2::new(-1, 0);
const RIGHT: IVec2 = IVec2::new(1, 0);

pub enum PlayerAction {
    Walk,
    Climb,
    Drop,
    Idle,
}

impl Level {
    pub fn check_valid(&self, pos: IVec2, action: PlayerAction, x_dir: i32) -> Option<Animation> {
        action
            .squares()
            .all(|square| !self.is_solid(pos + square * IVec2::new(x_dir, 1)))
            .then(|| action.animation())
            .filter(|anim| self.is_solid(pos + anim.final_offset(x_dir) + DOWN))
    }
}

pub struct Animation {
    final_offset: IVec2,
    pub duration: Duration,
    // returns
    pub func: Box<dyn Fn(f32) -> AnimationFrame + Send + Sync>,
}

pub struct AnimationFrame {
    pub state: PlayerAnimationState,
    offset: Vec2,
}

impl AnimationFrame {
    pub fn offset(&self, x_dir: i32) -> Vec2 {
        self.offset * Vec2::new(x_dir as f32, 1.)
    }
}

impl Animation {
    pub fn final_offset(&self, x_dir: i32) -> IVec2 {
        self.final_offset * IVec2::new(x_dir, 1)
    }
}

impl PlayerAction {
    // the squares in the order that they are touched
    pub fn squares(&self) -> impl Iterator<Item = IVec2> {
        match self {
            PlayerAction::Walk => vec![RIGHT],
            PlayerAction::Climb => vec![UP, UP + RIGHT],
            PlayerAction::Drop => vec![RIGHT, DOWN + RIGHT],
            PlayerAction::Idle => vec![],
        }
        .into_iter()
    }

    // animations
    pub fn animation(&self) -> Animation {
        match self {
            PlayerAction::Walk => Animation {
                final_offset: RIGHT,
                duration: Duration::from_secs_f32(0.8),
                func: Box::new(|f| AnimationFrame {
                    offset: Vec2::ZERO,
                    state: PlayerAnimationState::Walk(2 + (f * 10.0) as usize),
                }),
            },
            PlayerAction::Climb => Animation {
                final_offset: UP + RIGHT,
                duration: Duration::from_secs_f32(1.1),
                func: Box::new(|f| AnimationFrame {
                    offset: Vec2::ZERO,
                    state: PlayerAnimationState::Climb((f * 11.0) as usize),
                }),
            },
            PlayerAction::Drop => Animation {
                final_offset: DOWN + RIGHT,
                duration: Duration::from_secs_f32(1.0),
                func: Box::new(|f| AnimationFrame {
                    offset: Vec2::ZERO,
                    state: PlayerAnimationState::Drop((f * 12.0) as usize),
                }),
            },
            PlayerAction::Idle => Animation {
                final_offset: IVec2::ZERO,
                duration: Duration::from_secs_f32(1.0),
                func: Box::new(|f| AnimationFrame {
                    offset: Vec2::ZERO,
                    state: PlayerAnimationState::Idle((f * 4.0) as usize),
                }),
            },
        }
    }
}
