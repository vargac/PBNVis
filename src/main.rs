#![allow(dead_code)]
#![allow(unused_imports)]

use std::{io, env, process, fs, thread, time};
use std::f64::consts::PI;
use std::io::Write;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::rc::Rc;

use biodivine_lib_bdd::{Bdd, BddValuation, BddVariable};
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::
    {SymbolicContext, SymbolicAsyncGraph, GraphColoredVertices, GraphColors};

use nalgebra as na;
use na::{Vector3, UnitQuaternion, Translation3, Point3, Vector2, Point2};
use na::base::Scalar;
use kiss3d::window::Window;
use kiss3d::camera::{ArcBall, Camera};
use kiss3d::light::Light;
use kiss3d::resource::Mesh;
use kiss3d::event::{WindowEvent, MouseButton, Action};
use kiss3d::scene::SceneNode;

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

fn bdd_to_str(bdd: &Bdd) -> String {
    bdd.sat_valuations()
        .map(|val| val_to_str(val))
        .collect::<Vec<String>>()
        .join(", ")
}

fn vertices_to_str(vertices: &GraphColoredVertices) -> String {
    bdd_to_str(vertices.as_bdd())
}

fn get_explicit_bn(
        symb_graph: &SymbolicAsyncGraph,
        colors_bdd: &Bdd,
        unit_colors_bdd: &Bdd,
        color_i: u32) -> Option<BooleanNetwork> {
    let symb_context = symb_graph.symbolic_context();
    let mut i = 0;
    for color_valuation in colors_bdd.sat_valuations() {
        i += 1;
        let partial: Vec<(BddVariable, bool)> = symb_context
            .parameter_variables()
            .iter()
            .map(|v| (*v, color_valuation[*v]))
            .collect();
        let color_bdd = unit_colors_bdd.select(&partial);
        let color = GraphColors::new(color_bdd.clone(), symb_context);
        assert_eq!(1.0, color.approx_cardinality());

        if i == color_i {
            //println!("{}", bdd_to_str(&color_bdd));
            return Some(symb_graph.pick_witness(&color));
        }
    }
    return None;
}

fn read_color(min: u32, max: u32) -> u32 {
    let mut color_i: u32 = 0;
    while color_i == 0 {
        print!("Choose one: ");
        io::stdout().flush().unwrap();
        let mut color_str = String::new();
        color_i = match io::stdin().read_line(&mut color_str) {
            Ok(_) => match color_str.trim().parse() {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Error parsing \"{}\": {}", color_str.trim(), e);
                    0
                }
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                0
            }
        };
        if color_i < min || color_i > max {
            eprintln!("Invalid color");
            color_i = 0;
        }
    }
    color_i
}

fn symb_to_ord_graph(symb_graph: SymbolicAsyncGraph) -> Graph<String, ()> {
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
    graph
}

fn calc_depths<N, E>(dag: &Graph<N, E>) -> (Vec<usize>, Vec<usize>, usize) {
    let mut depths = vec![0; dag.node_count()];
    let mut max_depth = 0;
    let mut topo = Topo::new(&dag);
    while let Some(u) = topo.next(&dag) {
        for v in dag.neighbors(u) {
            if depths[u.index()] + 1 > depths[v.index()] {
                depths[v.index()] = depths[u.index()] + 1;
            }
        }
        if depths[u.index()] > max_depth {
            max_depth = depths[u.index()];
        }
    }

    let mut cnt_at_depth = vec![0; max_depth + 1];
    for depth in &depths {
        cnt_at_depth[*depth] += 1;
    }
    (depths, cnt_at_depth, max_depth)
}

fn add_aux_root(dag: &mut Graph<Vec<String>, ()>) -> NodeIndex {
    let mut roots = dag.node_indices().collect::<HashSet<NodeIndex>>();
    for node in dag.node_indices() {
        for neighbor in dag.neighbors(node) {
            roots.remove(&neighbor);
        }
    }
    let aux_root = dag.add_node(vec![String::from("root")]);
    for root in roots {
        dag.add_edge(aux_root, root, ());
    }
    aux_root
}

fn draw_dag(mut dag: Graph<Vec<String>, ()>) {
    let aux_root = add_aux_root(&mut dag);
    let (depths, cnt_at_depth, max_depth) = calc_depths(&dag);

    let mut window = Window::new_with_size("STG", 500, 500);
    let mut camera = ArcBall::new(
        Point3::new(5.0, 5.0, 5.0), Point3::new(0.0, 0.0, 0.0));
    window.set_light(Light::StickToCamera);

    let mut done_at_depth = vec![0; max_depth + 1];
    let mut scene_nodes = vec![SceneNode::new_empty(); dag.node_count()];
    let mut dfs = Dfs::new(&dag, aux_root);
    while let Some(node) = dfs.next(&dag) {
        let depth = depths[node.index()];
        let angle = PI as f32 / 2.0
            * done_at_depth[depth] as f32
            / cnt_at_depth[depth] as f32;
        let fdepth = depth as f32;
        let x = angle.cos() * fdepth;
        let y = -fdepth;
        let z = angle.sin() * fdepth;
        let size =
            (0.2 + (1.0 + dag[node].len() as f64).ln().atan() / PI) as f32;

        let mut sphere = window.add_sphere(size);
        sphere.data_mut().object_mut().unwrap().set_user_data(Box::new(size));
        sphere.set_color(1.0, fdepth / max_depth as f32, 0.0);
        sphere.append_translation(&Translation3::new(x, y, z));
        scene_nodes[node.index()] = sphere;

        done_at_depth[depth] += 1;
    }

    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut chosen: Option<(usize, Point3<f32>)> = None;
    while window.render_with_camera(&mut camera) {
        for edge_id in dag.edge_indices() {
            let (u, v) = dag.edge_endpoints(edge_id).unwrap();
            window.draw_line(
                &scene_node_pos(&scene_nodes[u.index()]),
                &scene_node_pos(&scene_nodes[v.index()]),
                &Point3::new(1.0, 0.0, 0.0));
        }
        for event in window.events().iter() {
            match event.value {
                WindowEvent::CursorPos(x, y, _) => {
                    cx = x;
                    cy = y;
                }
                WindowEvent::MouseButton(b, a, _) => {
                    if b == MouseButton::Button1 && a == Action::Press {
                        if let Some((i, c)) = chosen {
                            let n = &mut scene_nodes[i];
                            n.set_color(c.x, c.y, c.z);
                            chosen = None;
                        }
                        let win_size = window.size();
                        let (pos, dir) = camera.unproject(
                            &Point2::new(cx as f32, cy as f32),
                            &Vector2::new(win_size.x as f32, win_size.y as f32));
                        if let Some(i) = find_node(&pos, &dir, &scene_nodes) {
                            let n = &mut scene_nodes[i];
                            chosen = Some((i, *n.data().get_object().data().color()));
                            n.set_color(0.0, 0.0, 1.0);
                        }
                    }
                }
                _ => (),
            };
        }
    }
}

fn scene_node_pos(scene_node: &SceneNode) -> Point3<f32> {
    scene_node.data().local_translation().transform_point(&Point3::origin())
}

fn get_node_size(node: &SceneNode) -> f32 {
    *node.data().object().unwrap().data().user_data()
        .downcast_ref::<f32>().unwrap()
}

fn find_node<'a>(pos: &Point3<f32>,
                 dir: &Vector3<f32>,
                 nodes: &Vec<SceneNode>) -> Option<usize> {
    if let Some(nearest) = nodes
            .iter().enumerate()
            .filter(|(_, node)| get_node_size(node)
                           >= line_point_dst(pos, dir, &scene_node_pos(node)))
            .min_by_key(|(_, node)| (scene_node_pos(node) - pos).dot(&dir) as i32
                               - get_node_size(node) as i32) {
        return Some(nearest.0);
    }
    None
}

fn line_point_dst(pos: &Point3<f32>, dir: &Vector3<f32>, point: &Point3<f32>)
-> f32 {
    let dst_vec = point - pos - (point - pos).dot(&dir) / dir.dot(&dir) * dir;
    dst_vec.dot(&dst_vec).sqrt()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Use with one parameter -- path to the .aeon model");
        process::exit(1);
    }
    let buffer = fs::read_to_string(&args[1])
        .expect("Cannot read the file");

    let model = BooleanNetwork::try_from(buffer.as_str()).unwrap();
    let symb_graph = SymbolicAsyncGraph::new(model).unwrap();

    let unit_colors = symb_graph.mk_unit_colors();
    let unit_colors_bdd = unit_colors.as_bdd();
    let colors_partial_valuation: Vec<(BddVariable, bool)> = symb_graph
        .symbolic_context()
        .state_variables()
        .iter()
        .map(|v| (*v, true))
        .collect();
    let colors_bdd = unit_colors_bdd.select(&colors_partial_valuation);

    println!("Number of colors: {}", colors_bdd.cardinality());
    let color_i = read_color(1, colors_bdd.cardinality() as u32);

    let model =
        get_explicit_bn(&symb_graph, &colors_bdd, &unit_colors_bdd, color_i)
        .expect("Error getting explicit boolean network.");
    println!("Computed explicit boolean network");

    let graph = symb_to_ord_graph(SymbolicAsyncGraph::new(model).unwrap());
    println!("Computed STG");
    let condensed = condensation(graph, true);
    println!("Condensation of STG done");

    //println!("{:?}", Dot::with_config(
    //    &condensed.map(|u, _| depths[u.index()], |_, _| ()),
    //    &[Config::EdgeNoLabel]));
    
    draw_dag(condensed);
}
