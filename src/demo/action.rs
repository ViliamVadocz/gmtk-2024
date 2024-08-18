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
pub enum ScriptCommand {
    Walk,
    Climb,
    Drop,
    Idle,
    Turn,
    Jump,
    OpenBracket,
    CloseBracket,
}

impl ScriptCommand {
    pub fn get_resource(self, assets: &PlayerAssets) -> &AnimationResource {
        match self {
            ScriptCommand::Walk => &assets.walk,
            ScriptCommand::Climb => &assets.climb,
            ScriptCommand::Drop => &assets.drop,
            ScriptCommand::Idle => &assets.idle,
            ScriptCommand::Turn => &assets.turn,
            ScriptCommand::Jump => &assets.jump,
            // spaghetti
            ScriptCommand::OpenBracket => unreachable!(),
            ScriptCommand::CloseBracket => unreachable!(),
        }
    }

    pub fn try_from(c: char) -> Option<Self> {
        Some(match c.to_ascii_lowercase() {
            'w' => ScriptCommand::Walk,
            'c' => ScriptCommand::Climb,
            'd' => ScriptCommand::Drop,
            'i' => ScriptCommand::Idle,
            't' => ScriptCommand::Turn,
            'j' => ScriptCommand::Jump,
            '(' => ScriptCommand::OpenBracket,
            ')' => ScriptCommand::CloseBracket,
            _ => return None,
        })
    }
}

impl Level {
    pub fn animation_is_valid(&self, pos: IVec2, anim: &AnimationResource, x_dir: i32) -> bool {
        let squares = &mut anim.squares.iter().copied();
        let free = squares.all(|square| !self.is_solid(pos + square * IVec2::new(x_dir, 1)));
        free && self.is_solid(pos + anim.final_offset(x_dir) + DOWN)
    }
}
