#![allow(dead_code)]
#![allow(unused_imports)]

use std::io::Read;
use std::collections::{HashMap, HashSet};

use biodivine_lib_bdd::BddValuation;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::
    {SymbolicContext, SymbolicAsyncGraph, GraphColoredVertices};

use nalgebra as na;
use na::{Vector3, UnitQuaternion, Translation3};
use kiss3d::window::Window;
use kiss3d::light::Light;

use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::dot::{Dot, Config};
use petgraph::algo::{condensation, dijkstra};
use petgraph::visit::Dfs;
use petgraph::graph::node_index as n;
use petgraph::visit::depth_first_search;
use petgraph::visit::{DfsEvent, Control};


fn val_to_str(val: BddValuation) -> String {
    val.vector().iter()
        .map(|v| (*v as i32).to_string())
        .collect::<Vec<String>>()
        .join("")
}

fn vertices_to_str(vertices: &GraphColoredVertices) -> String {
    vertices.as_bdd().sat_valuations()
        .map(|val| val_to_str(val))
        .collect::<Vec<String>>()
        .join(", ")
}

fn main() {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer).unwrap();

    let model = BooleanNetwork::try_from(buffer.as_str()).unwrap();
    let symb_graph = SymbolicAsyncGraph::new(model).unwrap();
    let mut graph = Graph::<String, ()>::new();
    let mut symb_to_idx = HashMap::new();
    let symb_vertices_bdd =
        symb_graph.unit_colored_vertices().as_bdd();

    for symb_vertex in symb_vertices_bdd.sat_valuations() {
        symb_to_idx.insert(
            symb_vertex.clone(), graph.add_node(val_to_str(symb_vertex)));
    }

    for symb_vertex_fr in symb_vertices_bdd.sat_valuations() {
        let col_vecs_to = symb_graph.post(&GraphColoredVertices::new(
            Bdd::from(symb_vertex_fr.clone()), symb_graph.symbolic_context()));
        for symb_vertex_to in col_vecs_to.as_bdd().sat_valuations() {
            graph.add_edge(*symb_to_idx.get(&symb_vertex_fr).unwrap(),
                           *symb_to_idx.get(&symb_vertex_to).unwrap(), ());
        }
    }

    let mut condensed = condensation(graph, true);
    //println!("{:?}", Dot::with_config(&condensed, &[Config::EdgeNoLabel]));

    let mut roots = condensed.node_indices().collect::<HashSet<NodeIndex>>();
    for node in condensed.node_indices() {
        for neighbor in condensed.neighbors(node) {
            roots.remove(&neighbor);
        }
    }

    let aux_root = condensed.add_node(vec![String::from("root")]);
    for root in roots {
        condensed.add_edge(aux_root, root, ());
    }
    let heights = dijkstra(&condensed, aux_root, None, |_| -1 as i32);

    println!("{:?}", Dot::with_config(
        &condensed.map(|u, _| -heights.get(&u).unwrap(), |_, _| ()),
        &[Config::EdgeNoLabel]));

    let min_height = heights.values().min().unwrap();
    let mut cnt_at_height = vec![0; -min_height as usize + 1];
    for height in heights.values() {
        cnt_at_height[-*height as usize] += 1;
    }


    let mut window = Window::new_with_size("STG", 500, 500);

    let mut done_at_height = vec![0; -min_height as usize + 1];
    for (node, height) in heights {
        let mut cube = window.add_cube(0.9, 0.9, 0.9);
        cube.set_color(1.0, 0.0, 0.0);

        let angle = std::f32::consts::PI / 2.0
            * done_at_height[-height as usize] as f32
            / cnt_at_height[-height as usize] as f32;
        cube.append_translation(
            &Translation3::new(angle.cos() * -height as f32,
                               height as f32,
                               angle.sin() * -height as f32));

        done_at_height[-height as usize] += 1;
    }

    window.set_light(Light::StickToCamera);

//    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    while window.render() {
    }
}
