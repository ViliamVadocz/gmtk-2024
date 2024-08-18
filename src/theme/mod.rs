//! Reusable UI widgets & theming.

// Unused utilities may trigger this lints undesirably.
#![allow(dead_code)]

pub mod interaction;
pub mod palette;
mod widgets;

#[allow(unused_imports)]
pub mod prelude {
    pub use super::{
        interaction::{InteractionPalette, OnPress},
        palette as ui_palette,
        widgets::{Containers as _, Widgets as _},
    };
}

use bevy::prelude::*;
use bevy_simple_text_input::{TextInputPlugin, TextInputSystem};
use widgets::focus;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(interaction::plugin);
    app.add_plugins(TextInputPlugin);
    app.add_systems(Update, focus.before(TextInputSystem));
}
