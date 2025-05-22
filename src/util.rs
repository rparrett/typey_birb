use bevy::{prelude::*, render::primitives::Aabb};

pub fn collide_aabb(a: &Aabb, b: &Aabb) -> bool {
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

pub fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
