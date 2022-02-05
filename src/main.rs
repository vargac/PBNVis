#![allow(dead_code)]
//#![allow(unused_imports)]

mod drawing_engine;
mod graph;
mod cli;

use std::env;

use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;

use crate::drawing_engine::DrawingEngine;
use crate::graph::Stg;


fn main() {
    let args: Vec<String> = env::args().collect();
    let model = BooleanNetwork::try_from(cli::load_bn(&args).as_str()).unwrap();
    let variable_names = model.variables()
        .map(|id| String::from(model.get_variable_name(id)))
        .collect::<Vec<String>>();
    let symb_graph = SymbolicAsyncGraph::new(model).unwrap();

    let colors_bdd = graph::get_symb_colors(&symb_graph);
    println!("Number of colors: {}", colors_bdd.cardinality());
    let color_i = cli::read_color(1, colors_bdd.cardinality() as u32);

    let model = graph::get_explicit_bn(&symb_graph, &colors_bdd, color_i)
        .expect("Error getting explicit boolean network.");
    println!("Computed explicit boolean network");
    let symb_graph = SymbolicAsyncGraph::new(model).unwrap();

    let graph = graph::symb_to_ord_graph(symb_graph);
    println!("Computed STG");
    let stg = Stg::new(graph);
    println!("Computed condensed STG");
    
    let printer = graph::NodePrinter::new(variable_names);
    let mut engine = DrawingEngine::new(stg, printer);
    engine.init();
    while engine.update() {
    }
}
