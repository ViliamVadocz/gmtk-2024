use bevy::prelude::*;

use super::{
    animation::{AnimationResource, PlayerAssets},
    level::Level,
};

pub const UP: IVec2 = IVec2::new(0, 1);
pub const DOWN: IVec2 = IVec2::new(0, -1);
// pub const LEFT: IVec2 = IVec2::new(-1, 0);
pub const RIGHT: IVec2 = IVec2::new(1, 0);

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
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
    pub fn get_resource(self, assets: &PlayerAssets) -> Vec<&AnimationResource> {
        match self {
            ScriptCommand::Walk => vec![&assets.walk],
            ScriptCommand::Climb => vec![&assets.climb],
            ScriptCommand::Drop => vec![&assets.drop, &assets.drop2],
            ScriptCommand::Idle => vec![&assets.idle],
            ScriptCommand::Turn => vec![&assets.turn],
            ScriptCommand::Jump => vec![&assets.jump],
            ScriptCommand::CloseBracket => unreachable!(),
            ScriptCommand::OpenBracket => unreachable!(),
        }
    }
}

impl Level {
    pub fn check_valid(
        &self,
        pos: IVec2,
        action: ScriptCommand,
        x_dir: i32,
        assets: &PlayerAssets,
    ) -> Option<AnimationResource> {
        let anim = action.get_resource(assets);
        anim.into_iter()
            .find(|anim| {
                let mut squares = anim.squares.iter().copied();
                let free =
                    squares.all(|square| !self.is_solid(pos + square * IVec2::new(x_dir, 1)));
                free && self.is_solid(pos + anim.final_offset(x_dir) + DOWN)
            })
            .cloned()
    }
}
