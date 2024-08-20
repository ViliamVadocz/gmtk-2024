//! Spawn the main level.

use bevy::{
    ecs::system::RunSystemOnce,
    prelude::*,
    utils::{HashMap, HashSet},
};
// use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use super::{animation::PlayerAssets, obstacle::Obstacle, player::Player};
use crate::{
    asset_tracking::LoadResource,
    demo::{action::ScriptCommand, obstacle::SpawnObstacle},
    screens::Screen,
    AppSet,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(LdtkPlugin);
    app.insert_resource(LdtkSettings {
        level_background: LevelBackground::Nonexistent,
        ..default()
    });
    app.load_resource::<LevelAssets>();
    app.insert_resource(LevelSelection::index(0));
    app.register_ldtk_entity::<PlayerStartBundle>("PlayerStart");
    app.register_ldtk_entity::<CheckpointBundle>("Checkpoint");
    app.register_ldtk_entity::<HazardBundle>("Hazard");
    app.register_ldtk_int_cell::<WallBundle>(1);
    app.add_systems(Update, load_level.run_if(in_state(Screen::Gameplay)));

    app.insert_resource(WorldGrid {
        origin: Vec2::splat(8.),
        size: Vec2::splat(16.),
    });
    app.init_resource::<Level>();
    app.insert_resource(AnimationTick(Timer::from_seconds(0.2, TimerMode::Once)));

    app.add_event::<TickStart>();
    app.add_event::<Reset>();
    app.add_systems(Update, update_tick_timer.in_set(AppSet::TickTimers));
}

#[derive(Default, Bundle, LdtkEntity)]
struct PlayerStartBundle {
    player_start: PlayerStart,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct PlayerStart;

#[derive(Default, Bundle, LdtkEntity)]
struct CheckpointBundle {
    checkpoint: Checkpoint,
    #[grid_coords]
    grid_coords: GridCoords,
    #[with(CommandCount::from_field)]
    command_count: CommandCount,
    #[with(Unlock::from_field)]
    unlock: Unlock,
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct CommandCount(i32);

impl CommandCount {
    fn from_field(entity_instance: &EntityInstance) -> Self {
        Self(
            *entity_instance
                .get_int_field("CommandCount")
                .expect("expected entity to have non-nullable `CommandCount` int field"),
        )
    }
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct Unlock(Option<ScriptCommand>);

impl Unlock {
    fn from_field(entity_instance: &EntityInstance) -> Self {
        Self(
            entity_instance
                .get_maybe_enum_field("Unlock")
                .expect("expected entity to have nullable `Unlock` enum field")
                .as_ref()
                .map(|field| match field.as_ref() {
                    "Walk" => ScriptCommand::Walk,
                    "Climb" => ScriptCommand::Climb,
                    "Idle" => ScriptCommand::Idle,
                    "Jump" => ScriptCommand::Jump,
                    "Drop" => ScriptCommand::Drop,
                    "Turn" => ScriptCommand::Turn,
                    "Brackets" => ScriptCommand::OpenBracket,
                    x => panic!("unexpected `Unlock` enum variant: {x}"),
                }),
        )
    }
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct Checkpoint;

#[derive(Default, Bundle, LdtkEntity)]
struct HazardBundle {
    hazard: Hazard,
    #[grid_coords]
    grid_coords: GridCoords,
    #[with(MoveTo::from_field)]
    move_to: MoveTo,
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct Hazard;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct MoveTo(Option<IVec2>);

impl MoveTo {
    fn from_field(entity_instance: &EntityInstance) -> Self {
        Self(
            *entity_instance
                .get_maybe_point_field("MoveTo")
                .expect("expected entity to have nullable `MoveTo` point field"),
        )
    }
}

#[derive(Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: Wall,
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct Wall;

#[derive(Resource, Asset, Reflect, Clone)]
pub struct LevelAssets {
    #[dependency]
    pub ldtk_project: Handle<LdtkProject>,
}

impl LevelAssets {
    pub const PATH_LDTK: &'static str = "map.ldtk";
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ldtk_project: assets.load(LevelAssets::PATH_LDTK),
        }
    }
}

/// A [`Command`] to spawn the level.
pub fn spawn_level(world: &mut World) {
    world.run_system_once(|mut commands: Commands, level_assets: Res<LevelAssets>| {
        commands.spawn(LdtkWorldBundle {
            ldtk_handle: level_assets.ldtk_project.clone(),
            ..Default::default()
        });
    });
}

// System that checks level spawn and loads the relevant info.
fn load_level(
    mut commands: Commands,
    mut level: ResMut<Level>,

    mut level_events: EventReader<LevelEvent>,
    walls: Query<
        &GridCoords,
        (
            With<Wall>,
            Without<PlayerStart>,
            Without<Checkpoint>,
            Without<Hazard>,
        ),
    >,
    player_start: Query<
        &GridCoords,
        (
            With<PlayerStart>,
            Without<Wall>,
            Without<Checkpoint>,
            Without<Hazard>,
        ),
    >,
    checkpoints: Query<
        (&GridCoords, &Unlock, &CommandCount),
        (
            With<Checkpoint>,
            Without<Wall>,
            Without<PlayerStart>,
            Without<Hazard>,
        ),
    >,
    hazards: Query<
        (&GridCoords, &MoveTo),
        (
            With<Hazard>,
            Without<Wall>,
            Without<PlayerStart>,
            Without<Checkpoint>,
        ),
    >,
    player_assets: Res<PlayerAssets>,
    player: Query<(), With<Player>>,
    obstacles: Query<Entity, With<Obstacle>>,
) {
    for level_event in level_events.read() {
        if let LevelEvent::Spawned(_level_iid) = level_event {
            log::info!("Loading level.");

            let wall_locations = walls.iter().map(|p| IVec2::new(p.x, p.y)).collect();
            level.walls = wall_locations;

            // Get unlocks from level file.
            let unlocks = checkpoints
                .iter()
                .map(|(p, &Unlock(unlock), &CommandCount(x))| {
                    (IVec2::new(p.x, p.y), (unlock, x.max(0) as usize))
                })
                .collect();
            level.unlocks = unlocks;

            // Despawn previous hazards.
            for entity in obstacles.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // Spawn hazards.
            for (grid_coords, move_to) in hazards.iter() {
                const LEVEL_HEIGHT: i32 = 64; // TODO: Get this info from somewhere.
                                              // IDK why the exported position uses a different coordinate system than the
                                              // grid coords.
                let pos = IVec2::new(grid_coords.x, grid_coords.y);
                let dest = move_to
                    .0
                    .map(|p| IVec2::new(p.x, LEVEL_HEIGHT - p.y))
                    .unwrap_or(pos);
                let dir = dest - pos;
                commands.add(SpawnObstacle { pos, dir });
            }

            // Spawn player and set player start only once.
            if player.get_single().is_err() {
                // Set player start / last checkpoint.
                let player_start = player_start.single();
                level.last_checkpoint = IVec2::new(player_start.x, player_start.y);

                // Spawn player.
                commands.spawn((
                    Name::new("Player"),
                    Player,
                    SpriteBundle {
                        texture: player_assets.texture.clone(),
                        sprite: Sprite::default(),
                        // transform: Transform::from_scale(Vec2::splat(4.0).extend(1.0)),
                        ..Default::default()
                    },
                    GridTransform(level.get_spawn()),
                    NextGridTransform(level.get_spawn()),
                    TextureAtlas {
                        layout: player_assets.layout.clone(),
                        index: 0,
                    },
                    StateScoped(Screen::Gameplay),
                ));
            }
        }
    }
}

#[derive(Resource, Debug)]
pub struct Level {
    walls: HashSet<IVec2>,
    pub unlocks: HashMap<IVec2, (Option<ScriptCommand>, usize)>,
    pub unlocked: Vec<ScriptCommand>,
    pub command_count: usize,
    pub last_checkpoint: IVec2,
}

/// Temporary hardcoded level for testing.
impl Default for Level {
    fn default() -> Self {
        Self {
            // These will be set on level load.
            walls: HashSet::default(),
            unlocks: HashMap::default(),
            last_checkpoint: IVec2::default(),
            // Start with just `Walk` and 1 command count.
            unlocked: vec![ScriptCommand::Walk],
            command_count: 1,
        }
    }
}

impl Level {
    /// Check whether the position is solid terrain.
    pub fn is_solid(&self, pos: IVec2) -> bool {
        self.walls.contains(&pos)
    }

    /// Check whether the position is a checkpoint.
    pub fn is_checkpoint(&self, pos: IVec2) -> bool {
        self.unlocks.contains_key(&pos)
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
) {
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
