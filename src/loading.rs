use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_pipelines_ready::{PipelinesReady, PipelinesReadyPlugin};

use crate::{util::cleanup, AppState};

#[derive(AssetCollection, Resource)]
pub struct GltfAssets {
    #[asset(path = "bevybird_gold.glb#Scene0")]
    pub birb_gold: Handle<Scene>,
    #[asset(path = "bevybird.glb#Scene0")]
    pub birb: Handle<Scene>,
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "Amatic-Bold.ttf")]
    pub main: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "menu.ogg")]
    pub menu: Handle<AudioSource>,
    #[asset(path = "play.ogg")]
    pub game: Handle<AudioSource>,
    #[asset(path = "flap.ogg")]
    pub flap: Handle<AudioSource>,
    #[asset(path = "badflap.ogg")]
    pub badflap: Handle<AudioSource>,
    #[asset(path = "score.ogg")]
    pub score: Handle<AudioSource>,
    #[asset(path = "crash.ogg")]
    pub crash: Handle<AudioSource>,
    #[asset(path = "bump.ogg")]
    pub bump: Handle<AudioSource>,
}

#[cfg(not(target_arch = "wasm32"))]
const EXPECTED_PIPELINES: usize = 11;
#[cfg(target_arch = "wasm32")]
const EXPECTED_PIPELINES: usize = 9;

#[derive(Component)]
struct LoadingOnly;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::Loading).continue_to_state(AppState::Pipelines),
        );
        app.add_collection_to_loading_state::<_, GltfAssets>(AppState::Loading);
        app.add_collection_to_loading_state::<_, FontAssets>(AppState::Loading);
        app.add_collection_to_loading_state::<_, AudioAssets>(AppState::Loading);

        app.add_plugins(PipelinesReadyPlugin);

        app.add_systems(OnEnter(AppState::Loading), loading);

        app.add_systems(OnEnter(AppState::Pipelines), preload);
        app.add_systems(
            Update,
            check_pipelines
                .run_if(in_state(AppState::Pipelines))
                .run_if(resource_changed::<PipelinesReady>()),
        );
        app.add_systems(OnExit(AppState::Pipelines), cleanup::<LoadingOnly>);
    }
}

fn loading(mut commands: Commands) {
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Loading...",
                TextStyle {
                    font_size: 20.,
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(5.),
                left: Val::Px(5.),
                ..default()
            },
            z_index: ZIndex::Global(100),
            ..default()
        },
        LoadingOnly,
    ));
}

fn preload(mut commands: Commands, gltf_assets: Res<GltfAssets>) {
    commands.spawn((
        SceneBundle {
            scene: gltf_assets.birb.clone(),
            transform: Transform::from_scale(Vec3::splat(0.1)),
            ..default()
        },
        LoadingOnly,
    ));
    commands.spawn((
        SceneBundle {
            scene: gltf_assets.birb_gold.clone(),
            transform: Transform::from_scale(Vec3::splat(0.1)),
            ..default()
        },
        LoadingOnly,
    ));
}

fn check_pipelines(ready: Res<PipelinesReady>, mut next_state: ResMut<NextState<AppState>>) {
    info!("Pipelines: {}/{}", ready.get(), EXPECTED_PIPELINES);
    if ready.get() >= EXPECTED_PIPELINES {
        next_state.set(AppState::StartScreen);
    }
}
