//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

pub mod action;
pub mod animation;
pub mod editor;
pub mod level;
mod obstacle;
pub mod player;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        player::plugin,
        level::plugin,
        obstacle::plugin,
        editor::plugin,
    ));
}
