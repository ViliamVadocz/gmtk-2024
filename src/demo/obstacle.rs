use bevy::{
    ecs::{system::RunSystemOnce as _, world::Command},
    prelude::*,
};

use super::{animation::PlayerAssets, level::GridTransform};
use crate::{
    demo::level::{AnimationTick, NextGridTransform, Reset, TickStart, WorldGrid},
    screens::Screen,
    AppSet,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, movement.in_set(AppSet::Update));
}

/// A command to spawn the player character.
#[derive(Debug, Clone)]
pub struct SpawnObstacle {
    pub pos: IVec2,
    pub dir: IVec2,
}

impl Command for SpawnObstacle {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self, spawn_obstacle);
    }
}

#[derive(Component)]
pub struct Obstacle {
    dir: IVec2,
    spawn: SpawnObstacle,
}

fn spawn_obstacle(
    In(config): In<SpawnObstacle>,
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
) {
    log::info!("Hazard spawning {config:?}");
    commands.spawn((
        Name::new("Obstacle"),
        Obstacle {
            dir: config.dir,
            spawn: config.clone(),
        },
        SpriteBundle {
            texture: player_assets.hazard_texture.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)),
            sprite: Sprite::default(),
            ..Default::default()
        },
        GridTransform(config.pos),
        NextGridTransform(config.pos),
        TextureAtlas {
            layout: player_assets.hazard_layout.clone(),
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
        &mut TextureAtlas,
    )>,
    tick: Res<AnimationTick>,
    proj: Res<WorldGrid>,
    mut tick_start: EventReader<TickStart>,
    mut reset: EventReader<Reset>,
) {
    let reset = reset.read().count() != 0;
    let ticks = tick_start.read().count();
    for (mut grid, mut next_grid, mut world, mut obstacle, mut atlas) in &mut o {
        if ticks % 2 == 1 {
            next_grid.0 = grid.0 + obstacle.dir;
            obstacle.dir = -obstacle.dir;
        }
        if reset {
            obstacle.dir = obstacle.spawn.dir;
            grid.0 = obstacle.spawn.pos;
            next_grid.0 = obstacle.spawn.pos;
        }

        let old = grid.0.as_vec2();
        let new = next_grid.0.as_vec2();
        let pos = old.lerp(new, tick.0.fraction());
        world.translation = proj.project_to_world(pos).extend(world.translation.z);

        atlas.index = ((tick.0.fraction() * 4.) as usize).min(3);
    }
}
