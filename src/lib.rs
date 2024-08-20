mod asset_tracking;
pub mod audio;
mod demo;
#[cfg(feature = "dev")]
mod dev_tools;
mod screens;
mod theme;

use bevy::{
    asset::AssetMetaCheck,
    audio::{AudioPlugin, Volume},
    input::mouse::MouseWheel,
    prelude::*,
};
use screens::Screen;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Order new `AppStep` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSet::TickTimers,
                AppSet::RecordInput,
                AppSet::Update,
                AppSet::ApplyAnimation,
                AppSet::UpdateCamera,
            )
                .chain(),
        );

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);

        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "GMTK 2024".to_string(),
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(AudioPlugin {
                    global_volume: GlobalVolume {
                        volume: Volume::new(0.0),
                    },
                    ..default()
                }),
        );

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            demo::plugin,
            screens::plugin,
            theme::plugin,
        ));

        app.add_systems(Update, camera_zoom.run_if(in_state(Screen::Gameplay)));

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSet {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
    ApplyAnimation,
    UpdateCamera,
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2dBundle {
            projection: OrthographicProjection {
                scale: 0.25,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(16.0 * 15.0, 16.0 * 10.0, 100.0)),
            ..default()
        },
        // Render all UI to this camera.
        // Not strictly necessary since we only use one camera,
        // but if we don't use this component, our UI will disappear as soon
        // as we add another camera. This includes indirect ways of adding cameras like using
        // [ui node outlines](https://bevyengine.org/news/bevy-0-14/#ui-node-outline-gizmos)
        // for debugging. So it's good to have this here for future-proofing.
        IsDefaultUiCamera,
    ));
}

fn camera_zoom(
    mut evr_scroll: EventReader<MouseWheel>,
    mut query: Query<&mut OrthographicProjection, With<IsDefaultUiCamera>>,
) {
    let Ok(mut projection) = query.get_single_mut() else {
        return;
    };

    use bevy::input::mouse::MouseScrollUnit;
    for ev in evr_scroll.read() {
        let y_scroll = match ev.unit {
            MouseScrollUnit::Line => {
                ev.y // line units
            }
            MouseScrollUnit::Pixel => {
                ev.y // pixel units
            }
        };
        projection.scale = (projection.scale - y_scroll / 100.0).clamp(0.1, 1.0)
    }
}
