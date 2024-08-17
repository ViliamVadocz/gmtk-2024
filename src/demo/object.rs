use bevy::{ecs::system::RunSystemOnce, prelude::*};

use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<TreeAssets>();
}

#[derive(Component)]
struct Tree;

pub fn spawn_tree_command(world: &mut World) {
    world.run_system_once(spawn_tree);
}

#[derive(Resource, Asset, Reflect, Clone)]
struct TreeAssets {
    #[dependency]
    pine: Handle<Image>,
}

impl FromWorld for TreeAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            pine: assets.load("images/isometric-plant-pack/isometric tiles/pine-none01.png"),
        }
    }
}

fn spawn_tree(mut commands: Commands, tree_assets: Res<TreeAssets>) {
    commands.spawn((
        Tree,
        SpriteBundle {
            sprite: Sprite {
                anchor: bevy::sprite::Anchor::BottomCenter,
                ..default()
            },
            texture: tree_assets.pine.clone(),
            transform: Transform::from_scale(Vec2::splat(4.).extend(1.))
                .with_translation(Vec2::new(0., -200.).extend(0.)),
            ..default()
        },
    ));
}
