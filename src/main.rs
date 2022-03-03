use bevy::{prelude::*, render::primitives::Aabb};
use rand::{thread_rng, Rng};

mod typing;
mod ui;
mod words;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Playing,
    Dead,
}

// Components

#[derive(Component)]
struct Birb;

#[derive(Component)]
struct TargetPosition(Vec3);
#[derive(Component)]
struct CurrentRotationZ(f32);

#[derive(Clone, Debug)]
pub enum Action {
    BirbUp,
    BirbDown,
    NewWord(Entity),
    IncScore(u32),
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

struct ObstacleTimer(Timer);

#[derive(Default)]
struct Score(u32);

fn main() {
    App::new()
        .insert_resource(ObstacleTimer(Timer::from_seconds(5., true)))
        .init_resource::<Score>()
        .add_plugins(DefaultPlugins)
        .add_state(AppState::Playing)
        .add_plugin(crate::typing::TypingPlugin)
        .add_plugin(crate::ui::UiPlugin)
        .add_event::<Action>()
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::on_update(AppState::Playing)
                .with_system(movement)
                .with_system(collision)
                .with_system(obstacle_movement)
                .with_system(spawn_obstacle)
                .with_system(update_target_position)
                .with_system(update_score),
        )
        //.add_system(rotate)
        //.add_system(daylight_cycle)
        .run();
}

fn collide_aabb(a: &Aabb, b: &Aabb) -> bool {
    let a_min = a.min();
    let a_max = a.max();
    let b_min = b.min();
    let b_max = b.max();

    a_max.x > b_min.x
        && a_min.x < b_max.x
        && a_max.y > b_min.y
        && a_min.y < b_max.y
        && a_max.z > b_min.z
        && a_min.z < b_max.z
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
) {
    let (birb, transform) = birb_query.single();
    let mut birb = birb.clone();
    birb.center += transform.translation;

    for (score_aabb, transform, entity) in score_collider_query.iter() {
        let mut score_aabb = score_aabb.clone();
        score_aabb.center += transform.translation;

        if collide_aabb(&score_aabb, &birb) {
            commands.entity(entity).insert(Used);
            info!("score!");
            score.0 += 2
        }
    }
    for (obstacle_aabb, transform) in obstacle_collider_query.iter() {
        let mut obstacle_aabb = obstacle_aabb.clone();
        obstacle_aabb.center += transform.translation;

        if collide_aabb(&obstacle_aabb, &birb) {
            state.set(AppState::Dead).unwrap();
            info!("ded!");
        }
    }
}

fn spawn_obstacle(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ObstacleTimer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let mut rng = thread_rng();

    info!("spawning obstacle");

    let gap_start = rng.gen_range(0.1..2.5);
    let gap_size = 2.;

    let bottom: Mesh = shape::Box {
        min_x: -0.5,
        max_x: 0.5,
        min_y: 0.,
        max_y: gap_start,
        min_z: -0.5,
        max_z: 0.5,
    }
    .into();
    let top: Mesh = shape::Box {
        min_x: -0.5,
        max_x: 0.5,
        min_y: gap_start + gap_size,
        max_y: 20.,
        min_z: -0.5,
        max_z: 0.5,
    }
    .into();

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
        .spawn_bundle((Transform::from_xyz(20., 0., 0.), GlobalTransform::default()))
        .with_children(|parent| {
            parent
                .spawn()
                .insert(bottom.compute_aabb().unwrap())
                .insert_bundle(PbrBundle {
                    mesh: meshes.add(bottom).into(),
                    material: materials.add(Color::GREEN.into()),
                    ..Default::default()
                })
                .insert(ObstacleCollider);

            parent
                .spawn()
                .insert(top.compute_aabb().unwrap())
                .insert_bundle(PbrBundle {
                    mesh: meshes.add(top).into(),
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
) {
    let speed = 2.;

    for (entity, mut transform) in query.iter_mut() {
        transform.translation.x -= time.delta_seconds() * speed;
        if transform.translation.x < -20. {
            commands.entity(entity).despawn_recursive();
        }
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
        if dist < std::f32::EPSILON {
            // if we are not moving, seek middle rotation
            if rotation.0.abs() < std::f32::EPSILON {
                continue;
            }

            let rot = if rotation.0 < 0. {
                time.delta_seconds() * rot_speed_glide
            } else {
                time.delta_seconds() * -rot_speed_glide
            };
            rotation.0 += rot;
            transform.rotation = Quat::from_rotation_z(rotation.0);

            continue;
        }

        let dir = target.0 - transform.translation;

        let rot = if dir.y > 0. {
            time.delta_seconds() * rot_speed
        } else {
            time.delta_seconds() * -rot_speed
        };
        rotation.0 = (rotation.0 + rot).clamp(-0.5, 0.5);
        transform.rotation = Quat::from_rotation_z(rotation.0);

        let delta = dir.normalize() * time.delta_seconds() * speed;
        if dist < delta.length() {
            transform.translation = target.0;
        } else {
            transform.translation += delta;
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

fn update_target_position(mut events: EventReader<Action>, mut query: Query<&mut TargetPosition>) {
    for e in events.iter() {
        match e {
            Action::BirbUp => {
                for mut target in query.iter_mut() {
                    target.0.y += 0.25;
                }
            }
            Action::BirbDown => {
                for mut target in query.iter_mut() {
                    target.0.y -= 0.25;
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
    asset_server: Res<AssetServer>,
) {
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0., 5., 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 50.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });

    // birb
    commands
        .spawn_bundle((
            Transform::from_xyz(0., 1., 0.).with_scale(Vec3::splat(0.25)),
            GlobalTransform::default(),
            TargetPosition(Vec3::new(0., 1., 0.)),
            CurrentRotationZ(0.),
            Aabb {
                center: Vec3::splat(0.),
                half_extents: Vec3::splat(0.25),
            },
            Birb,
        ))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("bevybird.glb#Scene0"));
        });

    commands.spawn_bundle(DirectionalLightBundle {
        ..Default::default()
    });
}
