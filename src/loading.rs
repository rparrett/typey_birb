use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_pipelines_ready::{PipelinesReady, PipelinesReadyPlugin};

use crate::AppState;

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
const EXPECTED_PIPELINES: usize = 33;
#[cfg(target_arch = "wasm32")]
const EXPECTED_PIPELINES: usize = 10;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::LoadingAssets)
                .load_collection::<GltfAssets>()
                .load_collection::<FontAssets>()
                .load_collection::<AudioAssets>()
                .continue_to_state(AppState::LoadingPipelines),
        );

        app.add_plugins(PipelinesReadyPlugin);

        app.add_systems(Startup, setup_ui);

        app.add_systems(OnEnter(AppState::LoadingPipelines), preload);

        app.add_systems(
            Update,
            (
                print_pipelines.run_if(resource_changed::<PipelinesReady>),
                check_pipelines
                    .run_if(in_state(AppState::LoadingPipelines))
                    .run_if(resource_changed::<PipelinesReady>),
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
        StateScoped(AppState::LoadingPipelines),
    ));
}

fn preload(mut commands: Commands, gltf_assets: Res<GltfAssets>) {
    commands.spawn((
        SceneRoot(gltf_assets.birb.clone()),
        Transform::from_scale(Vec3::splat(0.1)),
        StateScoped(AppState::LoadingPipelines),
    ));
    commands.spawn((
        SceneRoot(gltf_assets.birb_gold.clone()),
        Transform::from_scale(Vec3::splat(0.1)),
        StateScoped(AppState::LoadingPipelines),
    ));
}

fn print_pipelines(ready: Res<PipelinesReady>) {
    info!("Pipelines Ready: {}/{}", ready.get(), EXPECTED_PIPELINES);
}

fn check_pipelines(ready: Res<PipelinesReady>, mut next_state: ResMut<NextState<AppState>>) {
    if ready.get() >= EXPECTED_PIPELINES {
        next_state.set(AppState::StartScreen);
    }
}
