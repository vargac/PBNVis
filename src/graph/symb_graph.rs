use std::collections::HashMap;

use biodivine_lib_bdd::{Bdd, BddValuation, BddVariable, BddVariableSet};
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::symbolic_async_graph::
    {SymbolicAsyncGraph, GraphColoredVertices, GraphColors};

use petgraph::Graph;


pub struct NodePrinter {
    vars: BddVariableSet,
}

impl NodePrinter {
    pub fn new(var_names: Vec<String>) -> NodePrinter {
        NodePrinter {
            vars: BddVariableSet::new(var_names.iter()
                    .map(|s| s.as_str()).collect::<Vec<&str>>().as_slice()),
        }
    }

    pub fn node_to_string(&self, node_label: &[String]) -> String {
        let node_bdd = node_label.iter()
            .map(|label| Bdd::from(BddValuation::new(label
                .as_str().chars()
                .map(|c| c == '1')
                .collect::<Vec<bool>>())))
            .reduce(|accum, item| accum.or(&item)).unwrap();
        node_bdd.to_boolean_expression(&self.vars).to_string()
    }
}

fn val_to_str(val: BddValuation) -> String {
    val.vector().iter()
        .map(|v| (*v as i32).to_string())
        .collect::<Vec<String>>()
        .join("")
}

pub fn get_explicit_bn(
        symb_graph: &SymbolicAsyncGraph,
        colors_bdd: &Bdd,
        color_i: u32) -> Option<BooleanNetwork> {
    let symb_context = symb_graph.symbolic_context();
    let unit_colors_bdd = symb_graph.unit_colors().as_bdd();
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
            return Some(symb_graph.pick_witness(&color));
        }
    }
    return None;
}

pub fn symb_to_ord_graph(symb_graph: SymbolicAsyncGraph) -> Graph<String, ()> {
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

pub fn get_symb_colors(symb_graph: &SymbolicAsyncGraph) -> Bdd {
    let unit_colors = symb_graph.mk_unit_colors();
    let unit_colors_bdd = unit_colors.as_bdd();
    let colors_partial_valuation: Vec<(BddVariable, bool)> = symb_graph
        .symbolic_context()
        .state_variables()
        .iter()
        .map(|v| (*v, true))
        .collect();
    unit_colors_bdd.select(&colors_partial_valuation)
}
