use bevy::prelude::*;

use super::{
    animation::{AnimationResource, PlayerAssets},
    level::Level,
};

pub const UP: IVec2 = IVec2::new(0, 1);
pub const DOWN: IVec2 = IVec2::new(0, -1);
// pub const LEFT: IVec2 = IVec2::new(-1, 0);
pub const RIGHT: IVec2 = IVec2::new(1, 0);

#[derive(Clone, Copy)]
pub enum PlayerAction {
    Walk,
    Climb,
    Drop,
    Idle,
    Turn,
    Jump,
}

impl PlayerAction {
    pub fn get_resource(self, assets: &PlayerAssets) -> &AnimationResource {
        match self {
            PlayerAction::Walk => &assets.walk,
            PlayerAction::Climb => &assets.climb,
            PlayerAction::Drop => &assets.drop,
            PlayerAction::Idle => &assets.idle,
            PlayerAction::Turn => &assets.turn,
            PlayerAction::Jump => &assets.jump,
        }
    }

    pub fn try_from(c: char) -> Option<Self> {
        Some(match c.to_ascii_lowercase() {
            'w' => PlayerAction::Walk,
            'c' => PlayerAction::Climb,
            'd' => PlayerAction::Drop,
            'i' => PlayerAction::Idle,
            't' => PlayerAction::Turn,
            'j' => PlayerAction::Jump,
            _ => return None,
        })
    }
}

impl Level {
    pub fn check_valid(
        &self,
        pos: IVec2,
        action: PlayerAction,
        x_dir: i32,
        assets: &PlayerAssets,
    ) -> Option<AnimationResource> {
        let anim = action.get_resource(assets);
        let squares = &mut anim.squares.iter().copied();
        let free = squares.all(|square| !self.is_solid(pos + square * IVec2::new(x_dir, 1)));
        (free && self.is_solid(pos + anim.final_offset(x_dir) + DOWN)).then(|| anim.clone())
    }
}
