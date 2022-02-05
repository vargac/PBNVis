mod drawing_utils;

use std::f32::consts::PI;

use nalgebra::{Translation3, Point3, Vector2, Point2};
use kiss3d::window::Window;
use kiss3d::camera::{ArcBall, Camera};
use kiss3d::light::Light;
use kiss3d::event::{WindowEvent, MouseButton, Action};
use kiss3d::scene::SceneNode;

use crate::graph::{Stg, NodePrinter};
use drawing_utils::*;


pub struct DrawingEngine {
    stg: Stg,
    window: Window,
    camera: ArcBall,
    scene_nodes: Vec<SceneNode>,
    last_cursor_pos: Point2<f32>,
    chosen_node: Option<(usize, Point3<f32>)>,
    node_printer: NodePrinter,
}

impl DrawingEngine {
    pub fn new(stg: Stg, node_printer: NodePrinter) -> DrawingEngine {
        let node_cnt = stg.node_count();
        DrawingEngine {
            stg: stg,
            window: Window::new_with_size("STG", 500, 500),
            camera: ArcBall::new(Point3::new(5.0, 5.0, 5.0), Point3::origin()),
            scene_nodes: vec![SceneNode::new_empty(); node_cnt],
            last_cursor_pos: Point2::origin(),
            chosen_node: None,
            node_printer: node_printer,
        }
    }

    pub fn init(&mut self) {
        self.window.set_light(Light::StickToCamera);
        self.stg.dfs_with_depth_info(|node, depth, b_percent, d_percent| {
            let angle = PI / 2.0 * b_percent;
            let fdepth = depth as f32;
            let flen = self.stg.underlying[node].len() as f32;
            let size = 0.2 + (1.0 + flen).ln().atan() / PI;

            let mut sphere = self.window.add_sphere(size);
            sphere.data_mut().object_mut().unwrap()
                .set_user_data(Box::new(size));
            sphere.set_color(1.0, d_percent, 0.0);
            sphere.append_translation(&Translation3::new(
                angle.cos() * fdepth, -fdepth, angle.sin() * fdepth));
            self.scene_nodes[node.index()] = sphere;
        });
    }

    pub fn update(&mut self) -> bool {
        self.draw_edges();
        for event in self.window.events().iter() {
            match event.value {
                WindowEvent::CursorPos(x, y, _) => {
                    self.last_cursor_pos = Point2::new(x as f32, y as f32);
                }
                WindowEvent::MouseButton(b, a, _) => {
                    if b == MouseButton::Button1 && a == Action::Press {
                        self.on_mouse_click();
                    }
                }
                _ => (),
            };
        }
        self.window.render_with_camera(&mut self.camera)
    }

    fn draw_edges(&mut self) {
        for edge_id in self.stg.red_edge_indices() {
            let (u, v) = self.stg.red_edge_endpoints(edge_id).unwrap();
            self.window.draw_line(
                &node_pos(&self.scene_nodes[u as usize]),
                &node_pos(&self.scene_nodes[v as usize]),
                &self.scene_nodes[u as usize]
                    .data().get_object().data().color());
        }
    }

    fn on_mouse_click(&mut self) {
        if let Some((index, color)) = self.chosen_node {
            let n = &mut self.scene_nodes[index];
            n.set_color(color.x, color.y, color.z);
            self.chosen_node = None;
        }
        let size = Vector2::new(self.window.width() as f32,
                                self.window.height() as f32);
        let (pos, dir) = self.camera.unproject(&self.last_cursor_pos, &size);

        if let Some(index) = find_node(&pos, &dir, &self.scene_nodes) {
            let n = &mut self.scene_nodes[index];
            self.chosen_node =
                Some((index, *n.data().get_object().data().color()));
            n.set_color(0.0, 0.0, 1.0);
            let node_label = self.stg.node_label(index);
            println!("Chosen (size {}): {}",
                node_label.len(),
                self.node_printer.node_to_string(node_label));
        }
    }
}
