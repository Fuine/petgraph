//! Graph algorithms.
//!
//! It is a goal to gradually migrate the algorithms to be based on graph traits
//! so that they are generally applicable. For now, most of these use only the
//! **Graph** type.

use std::collections::BinaryHeap;
use std::borrow::{Borrow};

use super::{
    Graph,
    Undirected,
    EdgeType,
    Outgoing,
    Incoming,
    Dfs,
};
use scored::MinScored;
use super::visit::{
    Reversed,
    Visitable,
    VisitMap,
    IntoNeighborsDirected,
    IntoNodeIdentifiers,
    IntoExternals,
    NodeIndexable,
    NodeCompactIndexable,
    IntoEdgeReferences,
    EdgeRef,
};
use super::unionfind::UnionFind;
use super::graph::{
    IndexType,
    GraphIndex,
};

pub use super::isomorphism::{
    is_isomorphic,
    is_isomorphic_matching,
};
pub use super::dijkstra::dijkstra;

/// [Generic] Return the number of connected components of the graph.
///
/// For a directed graph, this is the *weakly* connected components.
pub fn connected_components<G>(g: G) -> usize
    where G: NodeCompactIndexable + IntoEdgeReferences,
{
    let mut vertex_sets = UnionFind::new(g.node_bound());
    for edge in g.edge_references() {
        let (a, b) = (edge.source(), edge.target());

        // union the two vertices of the edge
        vertex_sets.union(G::to_index(a), G::to_index(b));
    }
    let mut labels = vertex_sets.into_labeling();
    labels.sort();
    labels.dedup();
    labels.len()
}


/// [Generic] Return `true` if the input graph contains a cycle.
///
/// Always treats the input graph as if undirected.
pub fn is_cyclic_undirected<G>(g: G) -> bool
    where G: NodeIndexable + IntoEdgeReferences
{
    let mut edge_sets = UnionFind::new(g.node_bound());
    for edge in g.edge_references() {
        let (a, b) = (edge.source(), edge.target());

        // union the two vertices of the edge
        //  -- if they were already the same, then we have a cycle
        if !edge_sets.union(G::to_index(a), G::to_index(b)) {
            return true
        }
    }
    false
}

/*
/// **Deprecated: Renamed to `is_cyclic_undirected`.**
pub fn is_cyclic<N, E, Ty, Ix>(g: &Graph<N, E, Ty, Ix>) -> bool
    where Ty: EdgeType,
          Ix: IndexType,
{
    is_cyclic_undirected(g)
}
*/

/// [Generic] Perform a topological sort of a directed graph `g`.
///
/// Visit each node in order (if it is part of a topological order).
///
/// You can pass `g` as either **&Graph** or **&mut Graph**, and it
/// will be passed on to the visitor closure.
#[inline]
fn toposort_generic<G, F>(g: G, mut visit: F)
    where G: IntoNeighborsDirected + IntoExternals + Visitable,
          F: FnMut(G, G::NodeId),
{
    let mut ordered = g.borrow().visit_map();
    let mut tovisit = Vec::new();

    // find all initial nodes
    tovisit.extend(g.externals(Incoming));

    // Take an unvisited element and find which of its neighbors are next
    while let Some(nix) = tovisit.pop() {
        if ordered.is_visited(&nix) {
            continue;
        }
        visit(g, nix.clone());
        ordered.visit(nix.clone());
        for neigh in g.neighbors_directed(nix, Outgoing) {
            // Look at each neighbor, and those that only have incoming edges
            // from the already ordered list, they are the next to visit.
            if g.neighbors_directed(neigh.clone(), Incoming)
                .all(|b| ordered.is_visited(&b)) {
                tovisit.push(neigh);
            }
        }
    }
}

/// [Generic] Return `true` if the input directed graph contains a cycle.
///
/// Using the topological sort algorithm.
pub fn is_cyclic_directed<G>(g: G) -> bool
    where G: IntoNodeIdentifiers + IntoNeighborsDirected + IntoExternals + Visitable,
{
    let mut n_ordered = 0;
    toposort_generic(g, |_, _| n_ordered += 1);
    n_ordered != g.node_count()
}

/// [Generic] Perform a topological sort of a directed graph.
///
/// Return a vector of nodes in topological order: each node is ordered
/// before its successors.
///
/// If the returned vec contains less than all the nodes of the graph, then
/// the graph was cyclic.
pub fn toposort<G>(g: G) -> Vec<G::NodeId>
    where G: IntoNodeIdentifiers + IntoNeighborsDirected + IntoExternals + Visitable,
{
    let mut order = Vec::with_capacity(g.node_count());
    toposort_generic(g, |_, ix| order.push(ix));
    order
}

/// [Generic] Compute the *strongly connected components* using Kosaraju's algorithm.
///
/// Return a vector where each element is an scc.
///
/// For an undirected graph, the sccs are simply the connected components.
pub fn scc<G>(g: G) -> Vec<Vec<G::NodeId>>
    where G: IntoNeighborsDirected + Visitable + IntoNodeIdentifiers,
{
    let mut dfs = Dfs::empty(&g);

    // First phase, reverse dfs pass, compute finishing times.
    // http://stackoverflow.com/a/26780899/161659
    let mut finished = g.visit_map();
    let mut finish_order = Vec::new();
    for i in g.node_identifiers() {
        if dfs.discovered.is_visited(&i) {
            continue
        }
        dfs.move_to(i);
        while let Some(nx) = dfs.stack.last().cloned() {
            if finished.visit(nx) {
                // push again to record finishing time
                dfs.stack.push(nx);
                dfs.next(Reversed(g)).unwrap();
            } else {
                dfs.stack.pop();
                finish_order.push(nx);
            }
        }
    }

    g.reset_map(&mut dfs.discovered);
    let mut sccs = Vec::new();

    // Second phase
    // Process in decreasing finishing time order
    for i in finish_order.into_iter().rev() {
        if dfs.discovered.is_visited(&i) {
            continue;
        }
        // Move to the leader node.
        dfs.move_to(i);
        //let leader = nindex;
        let mut scc = Vec::new();
        while let Some(nx) = dfs.next(g) {
            scc.push(nx);
        }
        sccs.push(scc);
    }
    sccs
}


/// [Graph] Compute a *minimum spanning tree* of a graph.
///
/// Treat the input graph as undirected.
///
/// Using Kruskal's algorithm with runtime **O(|E| log |E|)**. We actually
/// return a minimum spanning forest, i.e. a minimum spanning tree for each connected
/// component of the graph.
///
/// The resulting graph has all the vertices of the input graph (with identical node indices),
/// and **|V| - c** edges, where **c** is the number of connected components in `g`.
pub fn min_spanning_tree<N, E, Ty, Ix>(g: &Graph<N, E, Ty, Ix>)
    -> Graph<N, E, Undirected, Ix>
    where N: Clone,
          E: Clone + PartialOrd,
          Ty: EdgeType,
          Ix: IndexType,
{
    if g.node_count() == 0 {
        return Graph::with_capacity(0, 0)
    }

    // Create a mst skeleton by copying all nodes
    let mut mst = Graph::with_capacity(g.node_count(), g.node_count() - 1);
    for node in g.raw_nodes() {
        mst.add_node(node.weight.clone());
    }

    // Initially each vertex is its own disjoint subgraph, track the connectedness
    // of the pre-MST with a union & find datastructure.
    let mut subgraphs = UnionFind::new(g.node_count());

    let mut sort_edges = BinaryHeap::with_capacity(g.edge_count());
    for edge in g.raw_edges() {
        sort_edges.push(MinScored(edge.weight.clone(), (edge.source(), edge.target())));
    }

    // Kruskal's algorithm.
    // Algorithm is this:
    //
    // 1. Create a pre-MST with all the vertices and no edges.
    // 2. Repeat:
    //
    //  a. Remove the shortest edge from the original graph.
    //  b. If the edge connects two disjoint trees in the pre-MST,
    //     add the edge.
    while let Some(MinScored(score, (a, b))) = sort_edges.pop() {
        // check if the edge would connect two disjoint parts
        if subgraphs.union(a.index(), b.index()) {
            mst.add_edge(a, b, score);
        }
    }

    debug_assert!(mst.node_count() == g.node_count());
    debug_assert!(mst.edge_count() < g.node_count());
    mst
}
