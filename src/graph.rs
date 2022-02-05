mod symb_graph;

use std::collections::HashSet;

use petgraph::Graph;
use petgraph::adj::{List, EdgeIndex as AdjEdgeIndex,
                    EdgeIndices as AdjEdgeIndices, NodeIndex as AdjNodeIndex};
use petgraph::graph::{NodeIndex, DefaultIx};
use petgraph::visit::Dfs;
use petgraph::algo::{condensation, toposort};
use petgraph::algo::tred::
    {dag_to_toposorted_adjacency_list, dag_transitive_reduction_closure};

pub use symb_graph::*;


type DagType = Graph<Vec<String>, ()>;

pub struct Stg {
    pub underlying: DagType,
    trans_red: List<()>, // indexing using 'revmap'
    revmap: Vec<DefaultIx>,
    topo: Vec<NodeIndex>, // same indexing as underlying
    aux_root: NodeIndex,
}

impl Stg {
    pub fn new(graph: Graph<String, ()>) -> Self {
        Self::from_condensed(condensation(graph, true))
    }

    pub fn from_condensed(mut dag: DagType) -> Self {
        let topo = toposort(&dag, None).unwrap();
        let (res, revmap) =
            dag_to_toposorted_adjacency_list::<_, DefaultIx>(&dag, &topo);
        let (trans_red, _) = dag_transitive_reduction_closure(&res);
        let aux_root = Self::add_aux_root(&mut dag);

        Stg {
            underlying: dag,
            trans_red: trans_red,
            revmap: revmap,
            topo: topo,
            aux_root: aux_root,
        }
    }

    pub fn dfs_with_depth_info<F>(&self, mut fun: F)
    where F: FnMut(NodeIndex, usize, f32, f32) {
        let (depths, cnt_at_depth, max_depth) = self.calc_depths();
        let mut done_at_depth = vec![0; max_depth + 1];
        let mut dfs = Dfs::new(&self.underlying, self.aux_root);
        while let Some(node) = dfs.next(&self.underlying) {
            if node == self.aux_root {
                continue;
            }
            let d = depths[node.index()];
            let breadth_percent =
                done_at_depth[d] as f32 / cnt_at_depth[d] as f32;
            let depth_percent = d as f32 / max_depth as f32;
            fun(node, d, breadth_percent, depth_percent);
            done_at_depth[d] += 1;
        }
    }

    pub fn node_label(&self, node_index: usize) -> &Vec<String> {
        &self.underlying[NodeIndex::new(node_index)]
    }
    
    pub fn node_count(&self) -> usize {
        self.underlying.node_count() - 1
    }

    pub fn red_edge_indices(&self) -> AdjEdgeIndices<(), DefaultIx> {
        self.trans_red.edge_indices()
    }

    pub fn red_edge_endpoints(&self, e: AdjEdgeIndex)
    -> Option<(AdjNodeIndex, AdjNodeIndex)> {
        self.trans_red.edge_endpoints(e).map(
            |(u, v)| (self.revmap[u as usize], self.revmap[v as usize]))
    }

    fn add_aux_root(dag: &mut DagType) -> NodeIndex {
        let mut roots = dag.node_indices().collect::<HashSet<NodeIndex>>();
        for node in dag.node_indices() {
            for neighbor in dag.neighbors(node) {
                roots.remove(&neighbor);
            }
        }
        let aux_root = dag.add_node(vec!["".into()]);
        for root in roots {
            dag.add_edge(aux_root, root, ());
        }
        aux_root
    }

    fn calc_depths(&self) -> (Vec<usize>, Vec<usize>, usize) {
        let mut depths = vec![0; self.underlying.node_count() - 1];
        let mut max_depth = 0;
        for u in &self.topo {
            if *u == self.aux_root {
                continue;
            }
            for v in self.underlying.neighbors(*u) {
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
}
