//! Spawn the main level.

use bevy::{ecs::world::Command, prelude::*};

use crate::{demo::player::SpawnPlayer, AppSet};

pub(super) fn plugin(app: &mut App) {
    // No setup required for this plugin.
    // It's still good to have a function here so that we can add some setup
    // later if needed.
    app.add_systems(
        Update,
        (
            update_tick_timer.in_set(AppSet::TickTimers),
            propagate_grid_transform.in_set(AppSet::PropagateGridTransform),
        ),
    );
    app.insert_resource(WorldGrid {
        origin: Vec2::splat(0.),
        size: Vec2::splat(50.),
    });
    app.add_event::<NextTick>();
    app.insert_resource(GridTick(Timer::from_seconds(0.2, TimerMode::Once)));
    app.init_resource::<Level>();
}

/// A [`Command`] to spawn the level.
/// Functions that accept only `&mut World` as their parameter implement
/// [`Command`]. We use this style when a command requires no configuration.
pub fn spawn_level(world: &mut World) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    SpawnPlayer { max_speed: 400.0 }.apply(world);
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct Level {
    terrain: Vec<bool>,
    row_size: usize,
    spawn: (usize, usize),
    unlocks: [(usize, usize); 3],
}

/// Temporary hardcoded level for testing.
impl Default for Level {
    fn default() -> Self {
        let o = false;
        let x = true;
        #[rustfmt::skip]
        let terrain = vec![
            x, o, o, o, o, o, o, o, o, o, o, o, o, o, o, x,
            x, o, o, o, o, o, o, o, o, o, o, o, o, o, o, x,
            x, o, o, o, o, o, o, o, o, o, o, o, o, o, o, x,
            x, x, x, o, o, o, o, o, o, o, o, o, o, o, o, x,
            o, o, x, x, o, o, o, o, o, o, o, o, o, o, o, x,
            o, o, o, x, x, o, o, o, o, o, o, x, x, x, x, x,
            o, o, o, o, x, x, x, x, x, x, x, x, o, o, o, o,
        ];

        let spawn = (1, 6);
        let unlocks = [(1, 9), (2, 13), (4, 1)];

        Self {
            terrain,
            row_size: 16,
            spawn,
            unlocks,
        }
    }
}

impl Level {
    /// Check whether the position is solid terrain.
    pub fn is_solid(&self, pos: IVec2) -> bool {
        self.get_terrain(pos).unwrap_or_default()
    }

    fn get_terrain(&self, pos: IVec2) -> Option<bool> {
        self.terrain
            .get(self.row_size * usize::try_from(pos.y).ok()? + usize::try_from(pos.x).ok()?)
            .copied()
    }

    pub fn get_spawn(&self) -> IVec2 {
        IVec2::new(self.spawn.0 as i32, self.spawn.1 as i32)
    }
}

#[derive(Resource)]
pub struct WorldGrid {
    origin: Vec2,
    size: Vec2,
}

impl WorldGrid {
    pub fn project_to_world(&self, coord: IVec2) -> Vec2 {
        coord.as_vec2().mul_add(self.size, self.origin)
    }
}

#[derive(Component)]
pub struct GridTransform(pub IVec2);

#[derive(Component)]
pub struct OldGridTransform(pub Vec<IVec2>);

fn propagate_grid_transform(
    mut q: Query<(&mut Transform, &GridTransform, &OldGridTransform)>,
    grid: Res<WorldGrid>,
    tick: Res<GridTick>,
) {
    for (mut transform, new, old) in &mut q {
        let old = grid.project_to_world(*old.0.last().unwrap_or(&new.0));
        let new = grid.project_to_world(new.0);
        let interpolated = old.lerp(new, tick.0.fraction());
        transform.translation = interpolated.extend(transform.translation.z);
    }
}

#[derive(Resource)]
pub struct GridTick(pub Timer);

pub fn update_tick_timer(time: Res<Time>, mut tick: ResMut<GridTick>) {
    tick.0.tick(time.delta());
}

#[derive(Event)]
pub struct NextTick;
