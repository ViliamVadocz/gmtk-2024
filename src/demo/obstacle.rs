use bevy::{
    ecs::{system::RunSystemOnce as _, world::Command},
    prelude::*,
};

use super::{animation::PlayerAssets, level::GridTransform};
use crate::{
    demo::{
        action::{DOWN, UP},
        level::{AnimationTick, NextGridTransform, Reset, TickStart, WorldGrid},
    },
    screens::Screen,
    AppSet,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, movement.in_set(AppSet::Update));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

/// A command to spawn the player character.
#[derive(Debug, Clone)]
pub struct SpawnObstacle {
    pub pos: IVec2,
    pub going_up: bool,
}

impl Command for SpawnObstacle {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self, spawn_obstacle);
    }
}

#[derive(Component)]
pub struct Obstacle {
    going_up: bool,
    spawn: SpawnObstacle,
}

fn spawn_obstacle(
    In(config): In<SpawnObstacle>,
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
) {
    commands.spawn((
        Name::new("Obstacle"),
        Obstacle {
            going_up: config.going_up,
            spawn: config.clone(),
        },
        SpriteBundle {
            texture: player_assets.idle.texture.clone(),
            transform: Transform::from_scale(Vec2::splat(4.0).extend(1.0)),
            sprite: Sprite::default(),
            ..Default::default()
        },
        GridTransform(config.pos),
        NextGridTransform(config.pos),
        TextureAtlas {
            layout: player_assets.idle.atlas.clone(),
            index: 0,
        },
        StateScoped(Screen::Gameplay),
    ));
}

fn movement(
    mut o: Query<(
        &mut GridTransform,
        &mut NextGridTransform,
        &mut Transform,
        &mut Obstacle,
    )>,
    tick: Res<AnimationTick>,
    proj: Res<WorldGrid>,
    mut tick_start: EventReader<TickStart>,
    mut reset: EventReader<Reset>,
) {
    let reset = reset.read().count() != 0;
    let ticks = tick_start.read().count();
    for (mut grid, mut next_grid, mut world, mut obstacle) in &mut o {
        if ticks % 2 == 1 {
            next_grid.0 = match obstacle.going_up {
                true => grid.0 + UP,
                false => grid.0 + DOWN,
            };
            obstacle.going_up = !obstacle.going_up;
        }
        if reset {
            obstacle.going_up = obstacle.spawn.going_up;
            grid.0 = obstacle.spawn.pos;
            next_grid.0 = obstacle.spawn.pos;
        }

        let old = grid.0.as_vec2();
        let new = next_grid.0.as_vec2();
        let pos = old.lerp(new, tick.0.fraction());
        world.translation = proj.project_to_world(pos).extend(world.translation.z);
    }
}
