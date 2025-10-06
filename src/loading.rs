use bevy::prelude::*;
use bevy_pipelines_ready::{PipelinesReady, PipelinesReadyPlugin};

use crate::{
    asset_tracking::{LoadResource, ResourceHandles},
    AppState,
};

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct GltfAssets {
    pub birb_gold: Handle<Scene>,
    pub birb: Handle<Scene>,
}

impl FromWorld for GltfAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            birb_gold: assets.load("bevybird_gold.glb#Scene0"),
            birb: assets.load("bevybird.glb#Scene0"),
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct FontAssets {
    pub main: Handle<Font>,
}

impl FromWorld for FontAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            main: assets.load("Amatic-Bold.ttf"),
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct AudioAssets {
    pub menu: Handle<AudioSource>,
    pub game: Handle<AudioSource>,
    pub flap: Handle<AudioSource>,
    pub badflap: Handle<AudioSource>,
    pub score: Handle<AudioSource>,
    pub crash: Handle<AudioSource>,
    pub bump: Handle<AudioSource>,
}

impl FromWorld for AudioAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            menu: assets.load("menu.ogg"),
            game: assets.load("play.ogg"),
            flap: assets.load("flap.ogg"),
            badflap: assets.load("badflap.ogg"),
            score: assets.load("score.ogg"),
            crash: assets.load("crash.ogg"),
            bump: assets.load("bump.ogg"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
const EXPECTED_PIPELINES: usize = 39;
#[cfg(target_arch = "wasm32")]
const EXPECTED_PIPELINES: usize = 10;
// Bevy's framerate seems to take a few seconds to fully recover after
// pipelines are compiled when running in Firefox.
#[cfg(not(target_arch = "wasm32"))]
const ADDITIONAL_WAIT_FRAMES: u32 = 10;
#[cfg(target_arch = "wasm32")]
const ADDITIONAL_WAIT_FRAMES: u32 = 50;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PipelinesReadyPlugin);

        app.load_resource::<GltfAssets>();
        app.load_resource::<FontAssets>();
        app.load_resource::<AudioAssets>();

        app.add_systems(Startup, setup_ui);

        app.add_systems(OnEnter(AppState::LoadingPipelines), preload);

        app.add_systems(
            Update,
            (
                check_assets.run_if(in_state(AppState::LoadingAssets)),
                print_pipelines.run_if(resource_changed::<PipelinesReady>),
                check_pipelines.run_if(in_state(AppState::LoadingPipelines)),
            ),
        );
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Children::spawn(Spawn((
            Text::new("Loading"),
            TextFont {
                font_size: 16.,
                ..default()
            },
        ))),
        DespawnOnExit(AppState::LoadingPipelines),
    ));
}

fn preload(mut commands: Commands, gltf_assets: Res<GltfAssets>) {
    commands.spawn((
        SceneRoot(gltf_assets.birb.clone()),
        Transform::from_scale(Vec3::splat(0.1)).with_translation(Vec3::Y * -1.0),
        DespawnOnExit(AppState::LoadingPipelines),
    ));
    commands.spawn((
        SceneRoot(gltf_assets.birb_gold.clone()),
        Transform::from_scale(Vec3::splat(0.1)).with_translation(Vec3::Y * -1.0),
        DespawnOnExit(AppState::LoadingPipelines),
    ));
}

fn print_pipelines(ready: Res<PipelinesReady>) {
    info!("Pipelines Ready: {}/{}", ready.get(), EXPECTED_PIPELINES);
}

fn check_pipelines(
    ready: Res<PipelinesReady>,
    mut next_state: ResMut<NextState<AppState>>,
    mut frames_since_pipelines_ready: Local<u32>,
) {
    if ready.get() < EXPECTED_PIPELINES {
        return;
    }

    *frames_since_pipelines_ready += 1;

    if *frames_since_pipelines_ready < ADDITIONAL_WAIT_FRAMES {
        return;
    }

    next_state.set(AppState::StartScreen);
}

fn check_assets(
    resource_handles: Res<ResourceHandles>,
    mut next_state: ResMut<NextState<AppState>>,
    mut frames_since_assets_ready: Local<u32>,
) {
    if !resource_handles.is_all_done() {
        return;
    }

    // Force at least one frame to pass before moving on to pipeline compilation so that the UI
    // pipeline gets compiled separately and we display the loading text while the other pipelines
    // are compiling.

    *frames_since_assets_ready += 1;

    if *frames_since_assets_ready < 2 {
        return;
    }

    next_state.set(AppState::LoadingPipelines);
}
