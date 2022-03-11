use bevy::{
    audio::AudioSink,
    log::{Level, LogSettings},
    prelude::*,
    render::primitives::Aabb,
};
use bevy_asset_loader::{AssetCollection, AssetLoader};
#[cfg(feature = "inspector")]
use bevy_inspector_egui::WorldInspectorPlugin;
use rand::{thread_rng, Rng};
use util::collide_aabb;

mod cylinder;
mod typing;
mod ui;
mod util;
mod words;

#[derive(AssetCollection)]
struct GltfAssets {
    #[asset(path = "bevybird_gold.glb#Scene0")]
    birb_gold: Handle<Scene>,
    #[asset(path = "bevybird.glb#Scene0")]
    birb: Handle<Scene>,
}

#[derive(AssetCollection)]
struct FontAssets {
    #[asset(path = "Amatic-Bold.ttf")]
    main: Handle<Font>,
}

#[derive(AssetCollection)]
struct AudioAssets {
    #[asset(path = "menu.ogg")]
    menu: Handle<AudioSource>,
    #[asset(path = "play.ogg")]
    game: Handle<AudioSource>,
    #[asset(path = "flap.ogg")]
    flap: Handle<AudioSource>,
    #[asset(path = "badflap.ogg")]
    badflap: Handle<AudioSource>,
    #[asset(path = "score.ogg")]
    score: Handle<AudioSource>,
    #[asset(path = "crash.ogg")]
    crash: Handle<AudioSource>,
    #[asset(path = "bump.ogg")]
    bump: Handle<AudioSource>,
}

struct MusicController(Handle<AudioSink>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Loading,
    StartScreen,
    Playing,
    Paused,
    EndScreen,
}

// Components

#[derive(Component)]
struct Birb;
#[derive(Component)]
struct Rival;

#[derive(Component)]
struct TargetPosition(Vec3);
#[derive(Component)]
struct CurrentRotationZ(f32);

#[derive(Clone, Debug)]
pub enum Action {
    BadFlap,
    BirbUp,
    BirbDown,
    NewWord(Entity),
    IncScore(u32),
    Start,
    Retry,
}

#[derive(Component)]
struct Obstacle;
#[derive(Component)]
struct ScoreCollider;
#[derive(Component)]
struct ObstacleCollider;
#[derive(Component)]
struct Used;

// Resources
#[derive(Default)]
struct Score(u32);
#[derive(Default)]
struct DistanceToSpawn(f32);
struct ObstacleSpacing(f32);
impl Default for ObstacleSpacing {
    fn default() -> Self {
        Self(12.)
    }
}
struct Speed {
    current: f32,
    max: f32,
}
impl Default for Speed {
    fn default() -> Self {
        Self {
            current: 2.,
            max: 5.,
        }
    }
}
impl Speed {
    fn increase(&mut self, amt: f32) {
        self.current = (self.current + amt).min(self.max);
    }
}

fn main() {
    let mut app = App::new();

    AssetLoader::new(AppState::Loading)
        .continue_to_state(AppState::StartScreen)
        .with_collection::<GltfAssets>()
        .with_collection::<FontAssets>()
        .with_collection::<AudioAssets>()
        .build(&mut app);

    app.insert_resource(WindowDescriptor {
        title: "Typey Birb".into(),
        ..Default::default()
    })
    .insert_resource(ClearColor(Color::rgb_u8(177, 214, 222)))
    .insert_resource(LogSettings {
        level: Level::INFO,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins);

    #[cfg(feature = "inspector")]
    {
        app.add_plugin(WorldInspectorPlugin::new());
        app.add_system_set(SystemSet::on_update(AppState::Paused).with_system(pause));
        app.add_system_set(SystemSet::on_update(AppState::Playing).with_system(pause));
    }

    app.add_state(AppState::Loading);

    app.init_resource::<Score>()
        .init_resource::<Speed>()
        .init_resource::<DistanceToSpawn>()
        .init_resource::<ObstacleSpacing>()
        .add_event::<Action>();

    app.add_plugin(crate::typing::TypingPlugin)
        .add_plugin(crate::ui::UiPlugin);

    app.add_system_set(SystemSet::on_exit(AppState::Loading).with_system(setup))
        .add_system_set(
            SystemSet::on_enter(AppState::StartScreen)
                .with_system(spawn_birb)
                .with_system(start_screen_music),
        )
        .add_system_set(
            SystemSet::on_update(AppState::StartScreen).with_system(start_screen_movement),
        )
        .add_system_set(
            SystemSet::on_enter(AppState::Playing)
                .with_system(spawn_rival)
                .with_system(game_music),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Playing)
                .with_system(movement)
                .with_system(rival_movement)
                .with_system(collision)
                .with_system(obstacle_movement)
                .with_system(spawn_obstacle)
                .with_system(update_target_position)
                .with_system(update_score)
                .with_system(bad_flap_sound),
        )
        .add_system_set(
            SystemSet::on_update(AppState::StartScreen)
                .with_system(start_game)
                .with_system(bad_flap_sound),
        )
        .add_system_set(
            SystemSet::on_update(AppState::EndScreen)
                .with_system(rival_movement)
                .with_system(retry_game)
                .with_system(bad_flap_sound),
        )
        .add_system_set(SystemSet::on_exit(AppState::EndScreen).with_system(reset))
        .run();
}

fn pause(mut keyboard: ResMut<Input<KeyCode>>, mut state: ResMut<State<AppState>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match state.current() {
            AppState::Paused => {
                state.set(AppState::Playing).unwrap();
                keyboard.clear();
            }
            AppState::Playing => {
                state.set(AppState::Paused).unwrap();
                keyboard.clear();
            }
            _ => {}
        }
    }
}

fn reset(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Obstacle>, With<Birb>, With<Rival>)>>,
) {
    commands.insert_resource(Score::default());
    commands.insert_resource(Speed::default());
    commands.insert_resource(DistanceToSpawn::default());
    commands.insert_resource(ObstacleSpacing::default());

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn rival_movement(mut query: Query<&mut Transform, With<Rival>>, time: Res<Time>) {
    let speed = 5.;

    for mut transform in query.iter_mut() {
        if transform.translation.x < 3. {
            transform.translation.x += speed * time.delta_seconds();
        }

        let floaty = (time.seconds_since_startup() as f32).sin();
        transform.translation.y = 4. + floaty;

        transform.rotation = Quat::from_rotation_z((time.seconds_since_startup() as f32).cos() / 4.)
    }
}

fn spawn_rival(mut commands: Commands, gltf_assets: Res<GltfAssets>) {
    commands
        .spawn_bundle((
            Transform::from_xyz(-10., 4., 2.5).with_scale(Vec3::splat(0.25)),
            GlobalTransform::default(),
            CurrentRotationZ(0.),
            Rival,
        ))
        .with_children(|parent| {
            parent.spawn_scene(gltf_assets.birb_gold.clone());
        });
}

fn bad_flap_sound(
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
    mut events: EventReader<Action>,
) {
    for e in events.iter() {
        if let Action::BadFlap = e {
            audio.play(audio_assets.badflap.clone());
        }
    }
}

fn game_music(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    audio_sinks: Res<Assets<AudioSink>>,
    audio: Res<Audio>,
    controller: Option<Res<MusicController>>,
) {
    if let Some(controller) = controller {
        if let Some(sink) = audio_sinks.get(&controller.0) {
            sink.pause();
        }
    }
    let handle = audio_sinks.get_handle(audio.play_in_loop(audio_assets.game.clone()));
    commands.insert_resource(MusicController(handle));
}

fn start_screen_music(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    audio_sinks: Res<Assets<AudioSink>>,
    audio: Res<Audio>,
    controller: Option<Res<MusicController>>,
) {
    if let Some(controller) = controller {
        if let Some(sink) = audio_sinks.get(&controller.0) {
            sink.pause();
        }
    }
    let handle = audio_sinks.get_handle(audio.play_in_loop(audio_assets.menu.clone()));
    commands.insert_resource(MusicController(handle));
}

fn spawn_birb(mut commands: Commands, gltf_assets: Res<GltfAssets>) {
    let pos = Vec3::new(0., 3., 0.);

    commands
        .spawn_bundle((
            Transform::from_translation(pos).with_scale(Vec3::splat(0.25)),
            GlobalTransform::default(),
            TargetPosition(pos),
            CurrentRotationZ(0.),
            Aabb {
                center: Vec3::splat(0.),
                half_extents: Vec3::splat(0.25),
            },
            Birb,
        ))
        .with_children(|parent| {
            parent.spawn_scene(gltf_assets.birb.clone());
        });
}

fn collision(
    mut commands: Commands,
    birb_query: Query<(&Aabb, &Transform), With<Birb>>,
    score_collider_query: Query<
        (&Aabb, &GlobalTransform, Entity),
        (With<ScoreCollider>, Without<Used>),
    >,
    obstacle_collider_query: Query<(&Aabb, &GlobalTransform), With<ObstacleCollider>>,
    mut score: ResMut<Score>,
    mut state: ResMut<State<AppState>>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    let (birb, transform) = birb_query.single();
    let mut birb = birb.clone();
    birb.center += transform.translation;

    for (score_aabb, transform, entity) in score_collider_query.iter() {
        let mut score_aabb = score_aabb.clone();
        score_aabb.center += transform.translation;

        if collide_aabb(&score_aabb, &birb) {
            commands.entity(entity).insert(Used);
            score.0 += 2;

            audio.play(audio_assets.score.clone());
        }
    }
    for (obstacle_aabb, transform) in obstacle_collider_query.iter() {
        let mut obstacle_aabb = obstacle_aabb.clone();
        obstacle_aabb.center += transform.translation;

        if collide_aabb(&obstacle_aabb, &birb) {
            state.set(AppState::EndScreen).unwrap();

            audio.play(audio_assets.crash.clone());

            // it's possible to collide with the pipe and flange simultaneously
            // so we should only react to one game-ending collision.
            break;
        }
    }
}

fn spawn_obstacle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spacing: Res<ObstacleSpacing>,
    mut distance: ResMut<DistanceToSpawn>,
    mut speed: ResMut<Speed>,
) {
    if distance.0 > 0. {
        return;
    }

    distance.0 = spacing.0;

    speed.increase(0.1);

    let mut rng = thread_rng();

    let gap_start = rng.gen_range(0.1..4.6);
    let gap_size = 2.;

    let flange_height = 0.4;
    let flange_radius = 0.8;

    let bottom_height = gap_start;
    let bottom_cylinder = meshes.add(
        cylinder::Cylinder {
            radius: 0.75,
            resolution: 16,
            segments: 1,
            height: bottom_height,
        }
        .into(),
    );
    let bottom_y = bottom_height / 2.;

    let top_height = 10. - gap_start - gap_size;
    let top_cylinder = meshes.add(
        cylinder::Cylinder {
            radius: 0.75,
            resolution: 16,
            segments: 1,
            height: top_height,
        }
        .into(),
    );
    let top_y = gap_start + gap_size + top_height / 2.;

    let flange = meshes.add(
        cylinder::Cylinder {
            radius: flange_radius,
            resolution: 16,
            segments: 1,
            height: flange_height,
        }
        .into(),
    );
    let bottom_flange_y = gap_start - flange_height / 2.;
    let top_flange_y = gap_start + gap_size + flange_height / 2.;

    let middle: Mesh = shape::Box {
        min_x: -0.1,
        max_x: 1.0,
        min_y: gap_start,
        max_y: gap_start + gap_size,
        min_z: -0.5,
        max_z: 0.5,
    }
    .into();

    commands
        .spawn_bundle((Transform::from_xyz(30., 0., 0.), GlobalTransform::default()))
        .with_children(|parent| {
            parent
                .spawn()
                .insert_bundle(PbrBundle {
                    transform: Transform::from_xyz(0., bottom_y, 0.),
                    mesh: bottom_cylinder,
                    material: materials.add(Color::GREEN.into()),
                    ..Default::default()
                })
                .insert(ObstacleCollider);
            parent
                .spawn()
                .insert_bundle(PbrBundle {
                    transform: Transform::from_xyz(0., bottom_flange_y, 0.),
                    mesh: flange.clone(),
                    material: materials.add(Color::GREEN.into()),
                    ..Default::default()
                })
                .insert(ObstacleCollider);

            parent
                .spawn()
                .insert_bundle(PbrBundle {
                    transform: Transform::from_xyz(0., top_y, 0.),
                    mesh: top_cylinder,
                    material: materials.add(Color::GREEN.into()),
                    ..Default::default()
                })
                .insert(ObstacleCollider);
            parent
                .spawn()
                .insert_bundle(PbrBundle {
                    transform: Transform::from_xyz(0., top_flange_y, 0.),
                    mesh: flange.clone(),
                    material: materials.add(Color::GREEN.into()),
                    ..Default::default()
                })
                .insert(ObstacleCollider);

            parent
                .spawn()
                .insert_bundle((Transform::default(), GlobalTransform::default()))
                .insert(middle.compute_aabb().unwrap())
                .insert(ScoreCollider);
        })
        .insert(Obstacle);
}

fn obstacle_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform), With<Obstacle>>,
    time: Res<Time>,
    mut distance: ResMut<DistanceToSpawn>,
    speed: Res<Speed>,
) {
    let delta = time.delta_seconds() * speed.current;

    distance.0 -= delta;

    for (entity, mut transform) in query.iter_mut() {
        transform.translation.x -= delta;
        if transform.translation.x < -30. {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn start_screen_movement(mut query: Query<(&mut Transform, &mut TargetPosition)>, time: Res<Time>) {
    let speed = 1.0;
    let magnitude = 0.15;

    for (mut transform, mut target) in query.iter_mut() {
        let floaty = (time.seconds_since_startup() as f32 * speed).sin() * magnitude;
        transform.translation.y = 3. + floaty;
        target.0 = transform.translation;
    }
}

fn movement(
    mut query: Query<(&mut Transform, &mut CurrentRotationZ, &TargetPosition)>,
    time: Res<Time>,
) {
    let speed = 2.;
    let rot_speed = 2.;
    let rot_speed_glide = 1.;

    for (mut transform, mut rotation, target) in query.iter_mut() {
        let dist = target.0.distance(transform.translation);

        // if we are not moving, seek a neutral rotation
        if dist <= std::f32::EPSILON {
            if rotation.0.abs() <= std::f32::EPSILON {
                continue;
            }

            let delta = time.delta_seconds() * rot_speed_glide;

            if rotation.0 < 0. {
                rotation.0 = (rotation.0 + delta).min(0.);
            } else {
                rotation.0 = (rotation.0 - delta).max(0.);
            };

            transform.rotation = Quat::from_rotation_z(rotation.0);

            continue;
        }

        // otherwise, rotate with the direction of movement

        let dir = target.0 - transform.translation;

        let rot = if dir.y > 0. {
            time.delta_seconds() * rot_speed
        } else {
            time.delta_seconds() * -rot_speed
        };
        rotation.0 = (rotation.0 + rot).clamp(-0.5, 0.5);
        transform.rotation = Quat::from_rotation_z(rotation.0);

        // seek the target position

        let delta = dir.normalize() * time.delta_seconds() * speed;
        if dist < delta.length() {
            transform.translation = target.0;
        } else {
            transform.translation += delta;
        }
    }
}

fn retry_game(mut events: EventReader<Action>, mut state: ResMut<State<AppState>>) {
    for e in events.iter() {
        if let Action::Retry = e {
            state.set(AppState::StartScreen).unwrap();
        }
    }
}

fn start_game(mut events: EventReader<Action>, mut state: ResMut<State<AppState>>) {
    for e in events.iter() {
        if let Action::Start = e {
            state.set(AppState::Playing).unwrap();
        }
    }
}

fn update_score(mut events: EventReader<Action>, mut score: ResMut<Score>) {
    for e in events.iter() {
        if let Action::IncScore(inc) = e {
            score.0 += inc
        }
    }
}

fn update_target_position(
    mut events: EventReader<Action>,
    mut query: Query<&mut TargetPosition>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    let min = 0.5;
    let max = 6.5;

    for e in events.iter() {
        match e {
            Action::BirbUp => {
                for mut target in query.iter_mut() {
                    target.0.y += 0.25;
                    if target.0.y > max {
                        target.0.y = max;
                        audio.play(audio_assets.bump.clone());
                    } else {
                        audio.play(audio_assets.flap.clone());
                    }
                }
            }
            Action::BirbDown => {
                for mut target in query.iter_mut() {
                    target.0.y -= 0.25;
                    if target.0.y < min {
                        target.0.y = min;
                        audio.play(audio_assets.bump.clone());
                    } else {
                        audio.play(audio_assets.flap.clone());
                    }
                }
            }
            _ => {}
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(4.5, 5.8, 11.7).with_rotation(Quat::from_rotation_x(-0.211)),
        ..Default::default()
    });

    // ground
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(100., 1., 40.))),
        transform: Transform::from_xyz(0., -0.5, 0.),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });

    // directional 'sun' light
    const HALF_SIZE: f32 = 30.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..Default::default()
            },
            shadows_enabled: true,
            illuminance: 5000.,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4 / 2.)
                * Quat::from_rotation_y(std::f32::consts::PI / 8.),
            ..Default::default()
        },
        ..Default::default()
    });
}
