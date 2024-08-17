use bevy::prelude::*;

use super::level::Level;
pub enum PlayerAction {
    Left,
    Right,
    ClimbLeft,
    ClimbRight,
}

impl Level {
    pub fn check_valid(&self, pos: IVec2, action: PlayerAction) -> bool {
        const UP: IVec2 = IVec2::new(0, 1);
        const DOWN: IVec2 = IVec2::new(0, -1);
        const LEFT: IVec2 = IVec2::new(-1, 0);
        const RIGHT: IVec2 = IVec2::new(1, 0);
        match action {
            PlayerAction::Left => self.is_solid(pos + DOWN) && !self.is_solid(pos + LEFT),
            PlayerAction::Right => self.is_solid(pos + DOWN) && !self.is_solid(pos + RIGHT),
            PlayerAction::ClimbLeft => {
                self.is_solid(pos + LEFT)
                    && !self.is_solid(pos + UP)
                    && !self.is_solid(pos + UP + LEFT)
            }
            PlayerAction::ClimbRight => {
                self.is_solid(pos + RIGHT)
                    && !self.is_solid(pos + UP)
                    && !self.is_solid(pos + UP + RIGHT)
            }
        }
    }
}
