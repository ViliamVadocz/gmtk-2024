//! A credits screen that can be accessed from the title screen.

use bevy::prelude::*;

use crate::{screens::Screen, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Credits), spawn_credits_screen);
}

fn spawn_credits_screen(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Credits))
        .with_children(|children| {
            children.header("Programming");
            children.label("Hytak and Will");

            children.header("Animations and textures");
            children.label("Will");

            children.header("Level design");
            children.label("Will and Hytak");

            children.header("Other");
            children.label(
                "Bevy logo - All rights reserved by the Bevy Foundation. Permission granted for \
                 splash screen use when unmodified.",
            );
            children.label("Button SFX - CC0 by Jaszunio15");
            children.label("Music - CC BY 3.0 by Kevin MacLeod");

            children.button("Back").observe(enter_title_screen);
        });
}

fn enter_title_screen(_trigger: Trigger<OnPress>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
