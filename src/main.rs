// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    asset::AssetMetaCheck,
    color::palettes::css::{DEEP_PINK, LIME},
    math::{
        bounding::{Aabb3d, Bounded3d, BoundingVolume, IntersectsVolume},
        Vec3A, Vec3Swizzles,
    },
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
};
use bevy_simple_prefs::{Prefs, PrefsPlugin};

use loading::{AudioAssets, FontAssets, GltfAssets, LoadingPlugin};
use luck::NextGapBag;

mod ground;
mod loading;
mod luck;
mod typing;
mod ui;
mod words;

#[derive(Component)]
struct MusicController;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    LoadingAssets,
    LoadingPipelines,
    StartScreen,
    Playing,
    #[cfg(feature = "debug")]
    Paused,
    EndScreen,
}

#[derive(Component)]
struct Birb;
#[derive(Component)]
struct Rival;

#[derive(Component)]
struct TargetPosition(Vec3);
#[derive(Component)]
struct CurrentRotationZ(f32);
#[derive(Component)]
struct HitBox(Aabb3d);

#[derive(Clone, Debug, Event)]
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
#[derive(Resource, Default)]
struct Score(u32);
#[derive(Resource, Default)]
struct DistanceToSpawn(f32);
#[derive(Resource)]
struct ObstacleSpacing(f32);
impl Default for ObstacleSpacing {
    fn default() -> Self {
        Self(12.)
    }
}

#[derive(Resource)]
struct Speed {
    current: f32,
    max: f32,
}
impl Default for Speed {
    fn default() -> Self {
        Self {
            current: 2.,
            max: 4.4,
        }
    }
}
impl Speed {
    fn increase(&mut self, amt: f32) {
        self.current = (self.current + amt).min(self.max);
    }
}

#[derive(Prefs, Reflect, Default)]
struct ExamplePrefs {
    high_score: HighScore,
}
#[derive(Resource, Reflect, Clone, Default)]
struct HighScore(u32);

#[derive(Component)]
struct Orbit {
    angle: f32,
    distance: f32,
    origin: Vec2,
}

const BIRB_START_Y: f32 = 3.;

const BIRB_MIN_Y: f32 = 0.9;
const BIRB_MAX_Y: f32 = 6.3;

const GAP_SIZE: f32 = 2.;
const GAP_START_MIN_Y: f32 = 0.5;
const GAP_START_MAX_Y: f32 = 6.7 - GAP_SIZE;

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::srgb_u8(177, 214, 222)));

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Typey Birb".into(),
                    ..default()
                }),
                ..default()
            })
            // Workaround for Bevy attempting to load .meta files in wasm builds. On itch,
            // the CDN serves HTTP 403 errors instead of 404 when files don't exist, which
            // causes Bevy to break.
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    );

    app.init_state::<AppState>();
    app.enable_state_scoped_entities::<AppState>();

    app.add_plugins(LoadingPlugin);

    #[cfg(feature = "debug")]
    {
        app.add_plugins(bevy::remote::RemotePlugin::default());
        app.add_plugins(bevy::remote::http::RemoteHttpPlugin::default());
        app.add_systems(Update, pause.run_if(in_state(AppState::Paused)));
        app.add_systems(Update, pause.run_if(in_state(AppState::Playing)));
    }

    app.add_plugins(PrefsPlugin::<ExamplePrefs>::default());

    app.init_resource::<Score>()
        .init_resource::<Speed>()
        .init_resource::<DistanceToSpawn>()
        .init_resource::<ObstacleSpacing>()
        .insert_resource(NextGapBag::new(
            GAP_START_MIN_Y..GAP_START_MAX_Y,
            BIRB_START_Y,
        ))
        .add_event::<Action>();

    app.add_plugins(crate::typing::TypingPlugin)
        .add_plugins(crate::ui::UiPlugin)
        .add_plugins(crate::ground::GroundPlugin);

    app.add_systems(Startup, setup);

    app.add_systems(
        OnEnter(AppState::StartScreen),
        (spawn_birb, start_screen_music),
    );
    app.add_systems(
        Update,
        (start_screen_movement, start_game, bad_flap_sound).run_if(in_state(AppState::StartScreen)),
    );

    app.add_systems(OnExit(AppState::StartScreen), (spawn_rival, game_music));
    app.add_systems(
        Update,
        (
            movement,
            rival_movement,
            collision,
            obstacle_movement,
            spawn_obstacle,
            update_target_position,
            update_score,
            bad_flap_sound,
        )
            .run_if(in_state(AppState::Playing)),
    );

    #[cfg(feature = "debug")]
    app.add_systems(Update, debug_hitboxes);

    app.add_systems(
        Update,
        (retry_game, bad_flap_sound).run_if(in_state(AppState::EndScreen)),
    );

    app.add_systems(
        Update,
        rival_movement_end_screen.run_if(in_state(AppState::EndScreen)),
    );

    app.add_systems(OnEnter(AppState::EndScreen), save_high_score);

    app.add_systems(OnExit(AppState::EndScreen), reset);

    app.run();
}

#[cfg(feature = "debug")]
fn pause(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match state.get() {
            AppState::Paused => {
                next_state.set(AppState::Playing);
            }
            AppState::Playing => {
                next_state.set(AppState::Paused);
            }
            _ => {}
        }
        keyboard.clear();
    }
}

fn reset(mut commands: Commands) {
    commands.insert_resource(Score::default());
    commands.insert_resource(Speed::default());
    commands.insert_resource(DistanceToSpawn::default());
    commands.insert_resource(ObstacleSpacing::default());
}

fn rival_movement(mut query: Query<&mut Transform, With<Rival>>, time: Res<Time>) {
    let speed = 5.;

    for mut transform in query.iter_mut() {
        if transform.translation.x < 3. {
            transform.translation.x += speed * time.delta_secs();
        }

        let floaty = time.elapsed_secs().sin();
        transform.translation.y = 4. + floaty;

        transform.rotation = Quat::from_rotation_z(time.elapsed_secs().cos() / 4.)
    }
}

fn rival_movement_end_screen(
    mut query: Query<&mut Transform, (With<Rival>, Without<Obstacle>)>,
    obstacle_query: Query<&Transform, With<Obstacle>>,
    time: Res<Time>,
    mut maybe_orbit: Local<Option<Orbit>>,
) {
    let Ok(rival) = query.single() else {
        return;
    };

    if maybe_orbit.is_none() {
        let closest_obstacle = obstacle_query
            .iter()
            .map(|t| {
                (
                    t.translation.xz().distance_squared(rival.translation.xz()),
                    t.translation.xz(),
                )
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap();

        *maybe_orbit = Some(Orbit {
            angle: closest_obstacle.1.angle_to(rival.translation.xz()),
            origin: closest_obstacle.1,
            distance: closest_obstacle.0.sqrt(),
        })
    }
    let orbit = maybe_orbit.as_mut().unwrap();

    // Convert speed to angular speed.
    let speed = 5. / std::f32::consts::TAU / orbit.distance * 2.;
    let step = speed * time.delta_secs();

    for mut transform in query.iter_mut() {
        orbit.angle += step;

        let (sin, cos) = orbit.angle.sin_cos();

        transform.translation.x = sin * orbit.distance + orbit.origin.x;
        transform.translation.z = cos * orbit.distance + orbit.origin.y;

        // face direction of movement
        let y_rot = transform.rotation.to_euler(EulerRot::XYZ).1;
        let diff = orbit.angle - y_rot;

        let next_y_rot = if diff.abs() < step {
            y_rot + step * diff.signum()
        } else {
            orbit.angle
        };

        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            0.,
            next_y_rot,
            time.elapsed_secs().cos() / 4.,
        );

        let floaty = time.elapsed_secs().sin();
        transform.translation.y = 4. + floaty;
    }
}

fn spawn_rival(mut commands: Commands, gltf_assets: Res<GltfAssets>) {
    commands.spawn((
        SceneRoot(gltf_assets.birb_gold.clone()),
        Transform::from_xyz(-10., 4., 2.5).with_scale(Vec3::splat(0.25)),
        CurrentRotationZ(0.),
        Rival,
        Name::new("Rival"),
        StateScoped(AppState::EndScreen),
    ));
}

fn bad_flap_sound(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    mut events: EventReader<Action>,
) {
    for e in events.read() {
        if let Action::BadFlap = e {
            commands.spawn((
                AudioPlayer(audio_assets.badflap.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

fn game_music(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    existing_music_query: Query<(Entity, &AudioSink), With<MusicController>>,
) {
    for (entity, sink) in &existing_music_query {
        sink.pause();
        commands.entity(entity).despawn();
    }

    commands.spawn((
        AudioPlayer(audio_assets.game.clone()),
        PlaybackSettings::LOOP,
        MusicController,
        Name::new("GameMusic"),
    ));
}

fn start_screen_music(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    existing_music_query: Query<(Entity, &AudioSink), With<MusicController>>,
) {
    for (entity, sink) in &existing_music_query {
        sink.pause();
        commands.entity(entity).despawn();
    }

    commands.spawn((
        AudioPlayer(audio_assets.menu.clone()),
        PlaybackSettings::LOOP,
        MusicController,
        Name::new("StartScreenMusic"),
    ));
}

fn spawn_birb(mut commands: Commands, gltf_assets: Res<GltfAssets>) {
    let pos = Vec3::new(0., BIRB_START_Y, 0.);

    let hitbox = HitBox(Aabb3d::new(Vec3A::splat(0.), Vec3A::new(0.2, 0.3, 0.25)));

    commands.spawn((
        SceneRoot(gltf_assets.birb.clone()),
        Transform::from_translation(pos).with_scale(Vec3::splat(0.25)),
        TargetPosition(pos),
        CurrentRotationZ(0.),
        hitbox,
        Birb,
        Name::new("Birb"),
        StateScoped(AppState::EndScreen),
    ));
}

fn collision(
    mut commands: Commands,
    birb_query: Query<(&HitBox, &Transform), With<Birb>>,
    score_collider_query: Query<
        (&HitBox, &GlobalTransform, Entity),
        (With<ScoreCollider>, Without<Used>),
    >,
    obstacle_collider_query: Query<(&HitBox, &GlobalTransform), With<ObstacleCollider>>,
    mut score: ResMut<Score>,
    mut next_state: ResMut<NextState<AppState>>,
    audio_assets: Res<AudioAssets>,
) {
    let Ok((birb_hitbox, transform)) = birb_query.single() else {
        return;
    };

    let birb_aabb = birb_hitbox.0.translated_by(transform.translation);

    for (score_aabb, transform, entity) in score_collider_query.iter() {
        let score_aabb = score_aabb.0.translated_by(transform.translation());

        if score_aabb.intersects(&birb_aabb) {
            commands.entity(entity).insert(Used);
            score.0 += 2;

            commands.spawn((
                AudioPlayer(audio_assets.score.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }

    let mut hit_obstacle = false;
    for (obstacle_aabb, transform) in obstacle_collider_query.iter() {
        let obstacle_aabb = obstacle_aabb.0.translated_by(transform.translation());

        if obstacle_aabb.intersects(&birb_aabb) {
            hit_obstacle = true;
            break;
        }
    }

    if hit_obstacle {
        next_state.set(AppState::EndScreen);

        commands.spawn((
            AudioPlayer(audio_assets.crash.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

#[cfg(feature = "debug")]
fn debug_hitboxes(hitboxes: Query<(&HitBox, &GlobalTransform)>, mut gizmos: Gizmos) {
    for (hitbox, transform) in &hitboxes {
        let translated = hitbox.0.translated_by(transform.translation());
        gizmos.cuboid(
            Transform::from_scale((translated.half_size() * 2.0).into())
                .with_translation(transform.translation()),
            Color::WHITE,
        );
    }
}

fn spawn_obstacle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spacing: Res<ObstacleSpacing>,
    mut distance: ResMut<DistanceToSpawn>,
    mut speed: ResMut<Speed>,
    mut bag: ResMut<NextGapBag>,
) {
    if distance.0 > 0. {
        return;
    }

    distance.0 = spacing.0;

    speed.increase(0.1);

    let gap_start = bag.next().unwrap();

    const FLANGE_HEIGHT: f32 = 0.4;
    const FLANGE_RADIUS: f32 = 0.8;
    const CYLINDER_RADIUS: f32 = 0.75;
    const CYLINDER_RESOLUTION: u32 = 16;
    const CYLINDER_SEGMENTS: u32 = 1;

    let bottom_height = gap_start;
    let bottom_primitive = Cylinder {
        radius: CYLINDER_RADIUS,
        half_height: bottom_height / 2.,
    };
    let bottom_cylinder = meshes.add(
        bottom_primitive
            .mesh()
            .resolution(CYLINDER_RESOLUTION)
            .segments(CYLINDER_SEGMENTS)
            .build(),
    );
    let bottom_y = bottom_height / 2.;

    let top_height = 10. - gap_start - GAP_SIZE;
    let top_primitive = Cylinder {
        radius: CYLINDER_RADIUS,
        half_height: top_height / 2.,
    };
    let top_cylinder = meshes.add(
        top_primitive
            .mesh()
            .resolution(CYLINDER_RESOLUTION)
            .segments(CYLINDER_SEGMENTS)
            .build(),
    );
    let top_y = gap_start + GAP_SIZE + top_height / 2.;

    let flange_primitive = Cylinder {
        radius: FLANGE_RADIUS,
        half_height: FLANGE_HEIGHT / 2.,
    };
    let flange = meshes.add(
        flange_primitive
            .mesh()
            .resolution(CYLINDER_RESOLUTION)
            .segments(CYLINDER_SEGMENTS)
            .build(),
    );
    let bottom_flange_y = gap_start - FLANGE_HEIGHT / 2.;
    let top_flange_y = gap_start + GAP_SIZE + FLANGE_HEIGHT / 2.;

    let middle_primitive = Cuboid::new(1.0, GAP_SIZE, 1.0);
    let middle = meshes.add(middle_primitive);
    let middle_y = gap_start + GAP_SIZE / 2.;

    commands
        .spawn((
            Transform::from_xyz(38., 0., 0.),
            Visibility::default(),
            Obstacle,
            Name::new("Obstacle"),
            StateScoped(AppState::EndScreen),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(bottom_cylinder),
                MeshMaterial3d(materials.add(Color::from(LIME))),
                Transform::from_xyz(0., bottom_y, 0.),
                HitBox(bottom_primitive.aabb_3d(Isometry3d::IDENTITY)),
                ObstacleCollider,
            ));
            parent.spawn((
                Mesh3d(flange.clone()),
                MeshMaterial3d(materials.add(Color::from(LIME))),
                Transform::from_xyz(0., bottom_flange_y, 0.),
                HitBox(flange_primitive.aabb_3d(Isometry3d::IDENTITY)),
                ObstacleCollider,
            ));

            parent.spawn((
                Mesh3d(top_cylinder),
                MeshMaterial3d(materials.add(Color::from(LIME))),
                Transform::from_xyz(0., top_y, 0.),
                HitBox(top_primitive.aabb_3d(Isometry3d::IDENTITY)),
                ObstacleCollider,
            ));
            parent.spawn((
                Mesh3d(flange.clone()),
                MeshMaterial3d(materials.add(Color::from(LIME))),
                Transform::from_xyz(0., top_flange_y, 0.),
                HitBox(flange_primitive.aabb_3d(Isometry3d::IDENTITY)),
                ObstacleCollider,
            ));
            parent.spawn((
                Mesh3d(middle.clone()),
                MeshMaterial3d(materials.add(Color::from(DEEP_PINK.with_alpha(0.5)))),
                Transform::from_xyz(0., middle_y, 0.),
                HitBox(middle_primitive.aabb_3d(Isometry3d::IDENTITY)),
                Visibility::Hidden,
                ScoreCollider,
                Name::new("ScoreCollider"),
            ));
        });
}

fn obstacle_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform), With<Obstacle>>,
    time: Res<Time>,
    mut distance: ResMut<DistanceToSpawn>,
    speed: Res<Speed>,
) {
    let delta = time.delta_secs() * speed.current;

    distance.0 -= delta;

    for (entity, mut transform) in query.iter_mut() {
        transform.translation.x -= delta;
        if transform.translation.x < -30. {
            commands.entity(entity).despawn();
        }
    }
}

fn start_screen_movement(mut query: Query<(&mut Transform, &mut TargetPosition)>, time: Res<Time>) {
    let speed = 1.0;
    let magnitude = 0.15;

    for (mut transform, mut target) in query.iter_mut() {
        let float_y = (time.elapsed_secs() * speed).sin() * magnitude;
        transform.translation.y = 3. + float_y;
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
        if dist <= f32::EPSILON {
            if rotation.0.abs() <= f32::EPSILON {
                continue;
            }

            let delta = time.delta_secs() * rot_speed_glide;

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
            time.delta_secs() * rot_speed
        } else {
            time.delta_secs() * -rot_speed
        };
        rotation.0 = (rotation.0 + rot).clamp(-0.5, 0.5);
        transform.rotation = Quat::from_rotation_z(rotation.0);

        // seek the target position

        let delta = dir.normalize() * time.delta_secs() * speed;
        if dist < delta.length() {
            transform.translation = target.0;
        } else {
            transform.translation += delta;
        }
    }
}

fn retry_game(mut events: EventReader<Action>, mut next_state: ResMut<NextState<AppState>>) {
    for e in events.read() {
        if let Action::Retry = e {
            next_state.set(AppState::StartScreen);
        }
    }
}

fn start_game(mut events: EventReader<Action>, mut next_state: ResMut<NextState<AppState>>) {
    for e in events.read() {
        if let Action::Start = e {
            next_state.set(AppState::Playing);
        }
    }
}

fn update_score(mut events: EventReader<Action>, mut score: ResMut<Score>) {
    for e in events.read() {
        if let Action::IncScore(inc) = e {
            score.0 += inc
        }
    }
}

fn update_target_position(
    mut commands: Commands,
    mut events: EventReader<Action>,
    mut query: Query<&mut TargetPosition>,
    audio_assets: Res<AudioAssets>,
) {
    for e in events.read() {
        match e {
            Action::BirbUp => {
                for mut target in query.iter_mut() {
                    target.0.y += 0.25;
                    if target.0.y > BIRB_MAX_Y {
                        target.0.y = BIRB_MAX_Y;

                        commands.spawn((
                            AudioPlayer(audio_assets.bump.clone()),
                            PlaybackSettings::DESPAWN,
                        ));
                    } else {
                        commands.spawn((
                            AudioPlayer(audio_assets.flap.clone()),
                            PlaybackSettings::DESPAWN,
                        ));
                    }
                }
            }
            Action::BirbDown => {
                for mut target in query.iter_mut() {
                    target.0.y -= 0.25;
                    if target.0.y < BIRB_MIN_Y {
                        target.0.y = BIRB_MIN_Y;

                        commands.spawn((
                            AudioPlayer(audio_assets.bump.clone()),
                            PlaybackSettings::DESPAWN,
                        ));
                    } else {
                        commands.spawn((
                            AudioPlayer(audio_assets.flap.clone()),
                            PlaybackSettings::DESPAWN,
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

fn setup(mut commands: Commands) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(4.5, 5.8, 11.7).with_rotation(Quat::from_rotation_x(-0.211)),
    ));

    // directional 'sun' light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 1000.,
            ..default()
        },
        CascadeShadowConfigBuilder {
            maximum_distance: 40.,
            ..default()
        }
        .build(),
        // Rotate such that upcoming gaps can be spied from the shadows
        Transform {
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4 / 2.)
                * Quat::from_rotation_y(std::f32::consts::PI / 8.),
            ..default()
        },
    ));
}

fn save_high_score(score: Res<Score>, mut high_score: ResMut<HighScore>) {
    high_score.0 = score.0;
}
