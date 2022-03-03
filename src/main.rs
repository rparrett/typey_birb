use bevy::prelude::*;
use rand::{thread_rng, Rng};

mod typing;
mod ui;
mod words;

#[derive(Component)]
struct Birb;

#[derive(Component)]
struct TargetPosition(Vec3);

#[derive(Clone, Debug)]
pub enum Action {
    BirbUp,
    BirbDown,
    NewWord(Entity),
    IncScore(u32),
}

#[derive(Component)]
struct Obstacle;

// Resources

struct ObstacleTimer(Timer);

#[derive(Default)]
struct Score(u32);

fn main() {
    App::new()
        .insert_resource(ObstacleTimer(Timer::from_seconds(5., true)))
        .init_resource::<Score>()
        .add_plugins(DefaultPlugins)
        .add_plugin(crate::typing::TypingPlugin)
        .add_plugin(crate::ui::UiPlugin)
        .add_event::<Action>()
        .add_startup_system(setup)
        .add_system(movement)
        .add_system(update_target_position)
        .add_system(update_score)
        .add_system(obstacle_movement)
        .add_system(spawn_obstacle)
        //.add_system(rotate)
        //.add_system(daylight_cycle)
        .run();
}

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
struct Sun;

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

    commands
        .spawn_bundle((Transform::from_xyz(20., 0., 0.), GlobalTransform::default()))
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: meshes
                    .add(
                        shape::Box {
                            min_x: -0.5,
                            max_x: 0.5,
                            min_y: 0.,
                            max_y: gap_start,
                            min_z: -0.5,
                            max_z: 0.5,
                        }
                        .into(),
                    )
                    .into(),
                ..Default::default()
            });
            parent.spawn_bundle(PbrBundle {
                mesh: meshes
                    .add(
                        shape::Box {
                            min_x: -0.5,
                            max_x: 0.5,
                            min_y: gap_start + gap_size,
                            max_y: 20.,
                            min_z: -0.5,
                            max_z: 0.5,
                        }
                        .into(),
                    )
                    .into(),
                material: materials.add(Color::RED.into()),
                ..Default::default()
            });
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

fn movement(mut query: Query<(&mut Transform, &TargetPosition)>, time: Res<Time>) {
    let speed = 2.;

    for (mut transform, target) in query.iter_mut() {
        let dist = target.0.distance(transform.translation);
        if dist < std::f32::EPSILON {
            continue;
        }

        let dir = target.0 - transform.translation;

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
            //.with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
            GlobalTransform::default(),
            TargetPosition(Vec3::new(0., 1., 0.)),
            Birb,
        ))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("bevybird.glb#Scene0"));
        });

    commands
        .spawn_bundle(DirectionalLightBundle {
            ..Default::default()
        })
        .insert(Sun);
}
