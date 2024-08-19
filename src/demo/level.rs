//! Spawn the main level.

use bevy::{
    ecs::{system::RunSystemOnce, world::Command},
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
};
use bevy_ecs_tilemap::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    demo::{obstacle::SpawnObstacle, player::SpawnPlayer},
    AppSet,
};

pub(super) fn plugin(app: &mut App) {
    // No setup required for this plugin.
    // It's still good to have a function here so that we can add some setup
    // later if needed.
    app.add_systems(Update, update_tick_timer.in_set(AppSet::TickTimers));
    app.insert_resource(WorldGrid {
        origin: Vec2::splat(0.),
        size: Vec2::splat(64.),
    });
    app.add_event::<TickStart>();
    app.add_event::<Reset>();
    app.insert_resource(AnimationTick(Timer::from_seconds(0.2, TimerMode::Once)));
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

            for (i, &tile) in level.terrain.iter().enumerate() {
                let x = (i % level.row_size) as u32;
                let y = map_size.y - 1 - (i / level.row_size) as u32;
                let tile_pos = TilePos { x, y };

                let texture_index = match tile {
                    Tile::CheckPoint => 1,
                    _ => 0,
                };
                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        visible: TileVisible(matches!(tile, Tile::CheckPoint | Tile::Ground)),
                        texture_index: TileTextureIndex(texture_index),
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

    SpawnPlayer.apply(world);
    SpawnObstacle {
        pos: IVec2::new(8, 1),
        going_up: true,
    }
    .apply(world)
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct Level {
    terrain: Vec<Tile>,
    row_size: usize,
    unlocks: [(usize, usize); 3],
    pub last_checkpoint: IVec2,
}

#[derive(Reflect, Debug, Clone, Copy)]
enum Tile {
    Air,
    Ground,
    CheckPoint,
}

/// Temporary hardcoded level for testing.
impl Default for Level {
    fn default() -> Self {
        let o = Tile::Air;
        let x = Tile::Ground;
        let i = Tile::CheckPoint;
        #[rustfmt::skip]
        let terrain = vec![
            x, o, o, o, o, o, o, o, o, i, o, o, o, o, o, x,
            x, o, o, o, o, o, o, o, x, x, x, o, o, o, o, x,
            x, i, o, o, x, x, x, o, o, o, o, o, o, o, o, x,
            x, x, x, o, o, o, o, o, o, o, o, x, o, o, o, x,
            o, o, x, x, o, o, o, o, o, o, o, o, o, i, o, x,
            o, o, o, x, x, o, o, o, o, o, o, x, x, x, x, x,
            o, o, o, o, x, x, x, x, x, x, x, x, o, o, o, o,
        ];

        let spawn = (1, 6);
        let unlocks = [(1, 9), (2, 13), (4, 1)];

        Self {
            terrain,
            row_size: 16,
            last_checkpoint: IVec2::new(spawn.1, spawn.0),
            unlocks,
        }
    }
}

impl Level {
    /// Check whether the position is solid terrain.
    pub fn is_solid(&self, pos: IVec2) -> bool {
        self.get_terrain(pos)
            .map(|x| matches!(x, Tile::Ground))
            .unwrap_or_default()
    }

    /// Check whether the position is a checkpoint.
    pub fn is_checkpoint(&self, pos: IVec2) -> bool {
        self.get_terrain(pos)
            .map(|x| matches!(x, Tile::CheckPoint))
            .unwrap_or_default()
    }

    fn height(&self) -> usize {
        self.terrain.len() / self.row_size
    }

    fn get_terrain(&self, pos: IVec2) -> Option<Tile> {
        let y = usize::try_from(self.height() as i32 - 1 - pos.y).ok()?;
        self.terrain
            .get(self.row_size * y + usize::try_from(pos.x).ok()?)
            .copied()
    }

    pub fn get_spawn(&self) -> IVec2 {
        self.last_checkpoint
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

#[derive(Component)]
pub struct NextGridTransform(pub IVec2);

#[derive(Resource)]
pub struct AnimationTick(pub Timer);

pub fn update_tick_timer(
    time: Res<Time>,
    mut tick: ResMut<AnimationTick>,
    mut q: Query<(&mut GridTransform, &NextGridTransform)>,
    mut tick_start: EventReader<TickStart>,
) {
    if tick_start.read().count() != 0 {
        tick.0.reset();
    }

    tick.0.tick(time.delta());

    if tick.0.just_finished() {
        for (mut old, new) in &mut q {
            old.0 = new.0;
        }
    }
}

#[derive(Event)]
pub struct TickStart;

#[derive(Event)]
pub struct Reset;
