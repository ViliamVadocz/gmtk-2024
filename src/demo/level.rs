//! Spawn the main level.

use bevy::{
    ecs::{system::RunSystemOnce, world::Command},
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
};
use bevy_ecs_tilemap::prelude::*;

use super::player::PlayerState;
use crate::{asset_tracking::LoadResource, demo::player::SpawnPlayer, AppSet};

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
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Reflect, Clone)]
pub struct LevelAssets {
    #[dependency]
    pub tiles: Handle<Image>,
}

impl LevelAssets {
    pub const PATH_TILES: &'static str = "images/tiles.png";
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            tiles: assets.load_with_settings(
                LevelAssets::PATH_TILES,
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}

/// A [`Command`] to spawn the level.
pub fn spawn_level(world: &mut World) {
    world.run_system_once(
        |mut commands: Commands,
         world_grid: Res<WorldGrid>,
         level_assets: Res<LevelAssets>,
         level: Res<Level>| {
            let WorldGrid { origin, size } = *world_grid;
            let texture_handle = level_assets.tiles.clone();

            // https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/game_of_life.rs
            let map_size = TilemapSize {
                x: level.row_size as u32,
                y: (level.terrain.len() / level.row_size) as u32,
            };
            let mut tile_storage = TileStorage::empty(map_size);
            let tilemap_entity = commands.spawn_empty().id();

            for (i, &solid) in level.terrain.iter().enumerate() {
                let x = (i % level.row_size) as u32;
                let y = map_size.y - 1 - (i / level.row_size) as u32;
                let tile_pos = TilePos { x, y };
                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        visible: TileVisible(solid),
                        ..Default::default()
                    })
                    .id();
                tile_storage.set(&tile_pos, tile_entity);
            }

            let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
            let grid_size = tile_size.into();
            let map_type = TilemapType::Square;

            commands.entity(tilemap_entity).insert(TilemapBundle {
                grid_size,
                map_type,
                size: map_size,
                storage: tile_storage,
                texture: TilemapTexture::Single(texture_handle),
                tile_size,
                transform: Transform::from_scale(Vec3::new(
                    size.x / tile_size.x,
                    size.y / tile_size.y,
                    1.0,
                ))
                .with_translation(origin.extend(-1.0)),
                ..Default::default()
            });
        },
    );

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

    fn height(&self) -> usize {
        self.terrain.len() / self.row_size
    }

    fn get_terrain(&self, pos: IVec2) -> Option<bool> {
        let y = self.height() - 1 - usize::try_from(pos.y).ok()?;
        self.terrain
            .get(self.row_size * y + usize::try_from(pos.x).ok()?)
            .copied()
    }

    pub fn get_spawn(&self) -> IVec2 {
        IVec2::new(self.spawn.1 as i32, self.spawn.0 as i32)
    }
}

#[derive(Resource, Clone, Copy)]
pub struct WorldGrid {
    origin: Vec2,
    size: Vec2,
}

impl WorldGrid {
    pub fn project_to_world(&self, coord: Vec2) -> Vec2 {
        coord.mul_add(self.size, self.origin)
    }
}

#[derive(Component)]
pub struct GridTransform(pub IVec2);

fn propagate_grid_transform(
    mut q: Query<(
        &mut Transform,
        &GridTransform,
        &PlayerState,
        &mut TextureAtlas,
        &mut Sprite,
    )>,
    grid: Res<WorldGrid>,
    tick: Res<GridTick>,
) {
    for (mut transform, pos, state, mut atlas, mut sprite) in &mut q {
        if let Some(anim) = &state.animation {
            let frame = (anim.func)(tick.0.fraction());
            atlas.index = frame.state.get_atlas_index();
            let new = grid.project_to_world(pos.0.as_vec2() + frame.offset(state.x_dir));
            transform.translation = new.extend(transform.translation.z);
        } else {
            transform.translation = grid
                .project_to_world(pos.0.as_vec2())
                .extend(transform.translation.z);
        }
        sprite.flip_x = state.x_dir == -1;
    }
}

#[derive(Resource)]
pub struct GridTick(pub Timer);

pub fn update_tick_timer(time: Res<Time>, mut tick: ResMut<GridTick>) {
    tick.0.tick(time.delta());
}

#[derive(Event)]
pub struct NextTick;
