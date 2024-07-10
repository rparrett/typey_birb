use std::ops::Range;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use rand::{thread_rng, Rng};

use crate::{AppState, Speed};

const GROUND_LENGTH: f32 = 60.;
const GROUND_WIDTH: f32 = 40.;
const GROUND_VERTICES_X: u32 = 30;
const GROUND_VERTICES_Z: u32 = 20;

#[derive(Resource)]
pub struct GroundMesh(Handle<Mesh>);
impl FromWorld for GroundMesh {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();

        Self(meshes.add(ground_mesh(
            Vec2::new(GROUND_LENGTH, GROUND_WIDTH),
            UVec2::new(GROUND_VERTICES_X, GROUND_VERTICES_Z),
        )))
    }
}

#[derive(Resource)]
pub struct GroundMaterial(Handle<StandardMaterial>);
impl FromWorld for GroundMaterial {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self(materials.add(Color::srgb(0.63, 0.96, 0.26)))
    }
}

#[derive(Component)]
pub struct Ground;

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GroundMaterial>();
        app.init_resource::<GroundMesh>();
        app.add_systems(
            Update,
            (ground_movement, spawn_ground.after(ground_movement))
                .run_if(in_state(AppState::Playing)),
        );
        app.add_systems(OnEnter(AppState::LoadingPipelines), spawn_ground);
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
    mesh: Res<GroundMesh>,
    material: Res<GroundMaterial>,
    query: Query<&Transform, With<Ground>>,
) {
    // keep two ground chunks alive at all times

    if query.iter().len() >= 2 {
        return;
    }

    let max_x = query
        .iter()
        .map(|transform| transform.translation.x)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(-GROUND_LENGTH);

    commands.spawn((
        PbrBundle {
            mesh: mesh.0.clone(),
            transform: Transform::from_xyz(max_x + GROUND_LENGTH, 0.1, 0.),
            material: material.0.clone(),
            ..Default::default()
        },
        Ground,
        Name::new("Ground"),
    ));
}

pub fn ground_mesh(size: Vec2, num_vertices: UVec2) -> Mesh {
    let num_quads = num_vertices - UVec2::splat(1);
    let offset = size / -2.;

    let h_range: Range<f32> = -0.15..0.15;

    let mut rng = thread_rng();

    let mut positions: Vec<[f32; 3]> = vec![];
    let mut normals = vec![];
    let mut uvs = vec![];
    let mut indices = vec![];

    for x in 0..num_vertices.x {
        for z in 0..num_vertices.y {
            // Use the same height values for the first and last column so that
            // ground chunks placed next to each other are seamless.
            let h = if x == num_vertices.x - 1 {
                positions[z as usize][1]
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

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();
    mesh
}
