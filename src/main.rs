#![allow(dead_code)]
#![allow(unused_imports)]

use std::io::Read;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::rc::Rc;

use biodivine_lib_bdd::BddValuation;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::
    {SymbolicContext, SymbolicAsyncGraph, GraphColoredVertices};

use nalgebra as na;
use na::{Vector3, UnitQuaternion, Translation3, Point3};
use kiss3d::window::Window;
use kiss3d::light::Light;
use kiss3d::resource::Mesh;

use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::dot::{Dot, Config};
use petgraph::algo::{condensation, dijkstra};
use petgraph::visit::Dfs;
use petgraph::graph::node_index as n;
use petgraph::visit::depth_first_search;
use petgraph::visit::{DfsEvent, Control, Topo};


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

    let mut depths = vec![0; condensed.node_count()];
    let mut max_depth = 0;
    let mut topo = Topo::new(&condensed);
    while let Some(u) = topo.next(&condensed) {
        for v in condensed.neighbors(u) {
            if depths[u.index()] + 1 > depths[v.index()] {
                depths[v.index()] = depths[u.index()] + 1;
            }
        }
        if depths[u.index()] > max_depth {
            max_depth = depths[u.index()];
        }
    }

    println!("{:?}", Dot::with_config(
        &condensed.map(|u, _| depths[u.index()], |_, _| ()),
        &[Config::EdgeNoLabel]));

    let mut cnt_at_depth = vec![0; max_depth + 1];
    for depth in &depths {
        cnt_at_depth[*depth] += 1;
    }


    let mut window = Window::new_with_size("STG", 500, 500);
    window.set_light(Light::StickToCamera);

    let mut done_at_depth = vec![0; max_depth + 1];
    let mut node_pos = vec![Point3::new(0.0, 0.0, 0.0); condensed.node_count()];
    let mut dfs = Dfs::new(&condensed, aux_root);
    while let Some(node) = dfs.next(&condensed) {
        let depth = depths[node.index()];
        let angle = std::f32::consts::PI / 2.0
            * done_at_depth[depth] as f32
            / cnt_at_depth[depth] as f32;
        let fdepth = depth as f32;
        let x = angle.cos() * fdepth;
        let y = -fdepth;
        let z = angle.sin() * fdepth;

        node_pos[node.index()] = Point3::new(x, y, z);
        let mut sphere = window.add_sphere(0.5);
        sphere.set_color(1.0, fdepth / max_depth as f32, 0.0);
        sphere.append_translation(&Translation3::from(node_pos[node.index()]));

        done_at_depth[depth] += 1;
    }


    while window.render() {
        for edge_id in condensed.edge_indices() {
            let (u, v) = condensed.edge_endpoints(edge_id).unwrap();
            window.draw_line(&node_pos[u.index()], &node_pos[v.index()],
                &Point3::new(1.0, 0.0, 0.0));
        }
    }
}
