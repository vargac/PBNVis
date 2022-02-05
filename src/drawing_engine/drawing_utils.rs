use kiss3d::scene::SceneNode;
use nalgebra::{Point3, Vector3};


pub fn node_pos(node: &SceneNode) -> Point3<f32> {
    node.data().local_translation().transform_point(&Point3::origin())
}

pub fn get_node_size(node: &SceneNode) -> f32 {
    *node.data().object().unwrap().data().user_data()
        .downcast_ref::<f32>().unwrap()
}

pub fn find_node<'a>(pos: &Point3<f32>,
                 dir: &Vector3<f32>,
                 nodes: &Vec<SceneNode>) -> Option<usize> {
    if let Some(nearest) = nodes
            .iter()
            .map(|node| ray_sphere_dist(pos, dir, &node_pos(node),
                                        get_node_size(node)))
            .enumerate()
            .filter(|(_, dist)| dist.is_some())
            .min_by(|(_, d1), (_, d2)|
                d1.partial_cmp(d2).expect("Unexpected NaN")) {
        return Some(nearest.0);
    }
    None
}

fn line_point_dst(pos: &Point3<f32>,
                  dir: &Vector3<f32>,
                  point: &Point3<f32>) -> f32 {
    let dst_vec = point - pos
                  - (point - pos).dot(&dir) / dir.dot(&dir) * dir;
    dst_vec.dot(&dst_vec).sqrt()
}

fn ray_sphere_dist(pos: &Point3<f32>,
                   dir: &Vector3<f32>,
                   center: &Point3<f32>,
                   radius: f32) -> Option<f32> {
    let vec = center - pos;
    let a = dir.dot(&dir);
    let b = -2.0 * dir.dot(&vec);
    let c = vec.dot(&vec) - radius * radius;
    let d = b * b - 4.0 * a * c;
    if d < 0.0 {
        return None;
    }
    Some(f32::min(-b + d.sqrt() / (2.0 * a), -b - d.sqrt() / (2.0 * a)))
}
