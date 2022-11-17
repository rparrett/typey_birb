use std::ops::Range;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use rand::{thread_rng, Rng};

use crate::{AppState, Speed};

pub const GROUND_LENGTH: f32 = 60.;
const GROUND_WIDTH: f32 = 40.;
const GROUND_VERTICES_X: u32 = 30;
const GROUND_VERTICES_Z: u32 = 20;

#[derive(Component)]
pub struct Ground;

#[derive(Bundle)]
pub struct GroundBundle {
    #[bundle]
    pbr: PbrBundle,
    ground: Ground,
}

impl GroundBundle {
    pub fn new(
        x: f32,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) -> GroundBundle {
        Self {
            pbr: PbrBundle {
                mesh: meshes.add(ground_mesh(
                    Vec2::new(GROUND_LENGTH, GROUND_WIDTH),
                    UVec2::new(GROUND_VERTICES_X, GROUND_VERTICES_Z),
                )),
                transform: Transform::from_xyz(x, 0.1, 0.),
                material: materials.add(Color::rgb(0.63, 0.96, 0.26).into()),
                ..Default::default()
            },
            ground: Ground,
        }
    }
}

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::Playing)
                .with_system(ground_movement)
                .with_system(spawn_ground.after(ground_movement)),
        )
        .add_system_set(SystemSet::on_exit(AppState::Loading).with_system(setup));
    }
}

fn ground_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform), With<Ground>>,
    time: Res<Time>,
    speed: Res<Speed>,
) {
    let delta = time.delta_seconds() * speed.current;

    for (entity, mut transform) in query.iter_mut() {
        transform.translation.x -= delta;
        if transform.translation.x < -60. {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn spawn_ground(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&Transform, With<Ground>>,
) {
    // keep two ground chunks alive at all times

    if query.iter().count() >= 2 {
        return;
    }

    let max_x = query
        .iter()
        .max_by(|a, b| a.translation.x.partial_cmp(&b.translation.x).unwrap())
        .unwrap()
        .translation
        .x;

    commands.spawn(GroundBundle::new(max_x + GROUND_LENGTH, meshes, materials));
}

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(GroundBundle::new(0., meshes, materials));
}

pub fn ground_mesh(size: Vec2, num_vertices: UVec2) -> Mesh {
    let num_quads = num_vertices - UVec2::splat(1);
    let offset = size / -2.;

    let h_range: Range<f32> = -0.1..0.1;

    let mut rng = thread_rng();

    let mut positions = vec![];
    let mut normals = vec![];
    let mut uvs = vec![];
    let mut indices = vec![];

    for x in 0..num_vertices.x {
        for z in 0..num_vertices.y {
            let h = if x == 0 || x == num_vertices.x - 1 {
                0.0
            } else {
                rng.gen_range(h_range.clone())
            };

            positions.push([
                offset.x + x as f32 / num_quads.x as f32 * size.x,
                h,
                offset.y + z as f32 / num_quads.y as f32 * size.y,
            ]);
            normals.push([0., 1., 0.]);
            uvs.push([0., 0.]);
        }
    }

    for x in 0..num_quads.x {
        for z in 0..num_quads.y {
            let i = x * num_vertices.y + z;

            indices.extend_from_slice(&[
                i,
                i + 1,
                i + num_vertices.y,
                i + num_vertices.y,
                i + 1,
                i + num_vertices.y + 1,
            ]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();
    mesh
}
