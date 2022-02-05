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
            .iter().enumerate()
            .filter(|(_, node)| get_node_size(node)
                                >= line_point_dst(pos, dir,
                                                  &node_pos(node)))
            .min_by_key(|(_, node)| ((node_pos(node) - pos).dot(&dir)
                                     - get_node_size(node)) as i32) {
        return Some(nearest.0);
    }
    None
}

pub fn line_point_dst(pos: &Point3<f32>,
                      dir: &Vector3<f32>,
                      point: &Point3<f32>) -> f32 {
    let dst_vec = point - pos
                  - (point - pos).dot(&dir) / dir.dot(&dir) * dir;
    dst_vec.dot(&dst_vec).sqrt()
}
