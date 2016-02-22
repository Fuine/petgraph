//! Graph visitor algorithms.
//!

use fixedbitset::FixedBitSet;
use std::collections::{
    HashSet,
    VecDeque,
};
use std::hash::Hash;

use super::{
    graphmap,
    graph,
    EdgeType,
    EdgeDirection,
    Graph,
    GraphMap,
    Incoming,
};

use graph::{
    IndexType,
    NodeIndex,
};
#[cfg(feature = "stable_graph")]
use graph::stable::StableGraph;

use graphmap::{
    NodeTrait,
};

/// Base graph trait
pub trait GraphBase {
    type NodeId: Copy;
    type EdgeId: Copy;
}

impl<'a, G> GraphBase for &'a G where G: GraphBase {
    type NodeId = G::NodeId;
    type EdgeId = G::EdgeId;
}

/// A copyable reference to a graph.
pub trait GraphRef : Copy + GraphBase { }

impl<'a, G> GraphRef for &'a G where G: GraphBase { }

impl<G: GraphBase> GraphBase for Reversed<G> {
    type NodeId = G::NodeId;
    type EdgeId = G::EdgeId;
}

impl<G: GraphRef> GraphRef for Reversed<G> { }



/// **Deprecated**
///
/// NeighborIter gives access to the neighbors iterator.
pub trait NeighborIter<'a> : GraphBase {
    type Iter: Iterator<Item=Self::NodeId>;

    /// Return an iterator that visits all neighbors of the node **n**.
    fn neighbors(&'a self, n: Self::NodeId) -> Self::Iter;
}

impl<'a, N, E: 'a, Ty, Ix> NeighborIter<'a> for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Iter = graph::Neighbors<'a, E, Ix>;
    fn neighbors(&'a self, n: graph::NodeIndex<Ix>) -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors(self, n)
    }
}

#[cfg(feature = "stable_graph")]
impl<'a, N, E: 'a, Ty, Ix> NeighborIter<'a> for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Iter = graph::stable::Neighbors<'a, E, Ix>;
    fn neighbors(&'a self, n: graph::NodeIndex<Ix>)
        -> graph::stable::Neighbors<'a, E, Ix>
    {
        StableGraph::neighbors(self, n)
    }
}

#[cfg(feature = "stable_graph")]
impl<'a, N, E: 'a, Ty, Ix> IntoNeighbors for &'a StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Neighbors = graph::stable::Neighbors<'a, E, Ix>;
    fn neighbors(self, n: Self::NodeId) -> Self::Neighbors {
        (*self).neighbors(n)
    }
}


impl<'a, N: 'a, E> NeighborIter<'a> for GraphMap<N, E>
where N: Copy + Ord + Hash
{
    type Iter = graphmap::Neighbors<'a, N>;
    fn neighbors(&'a self, n: N) -> graphmap::Neighbors<'a, N>
    {
        GraphMap::neighbors(self, n)
    }
}

impl<'a, N: 'a, E> IntoNeighbors for &'a GraphMap<N, E>
    where N: Copy + Ord + Hash
{
    type Neighbors = graphmap::Neighbors<'a, N>;
    fn neighbors(self, n: N) -> graphmap::Neighbors<'a, N>
    {
        GraphMap::neighbors(self, n)
    }
}

/// Wrapper type for walking the graph as if it is undirected
#[derive(Copy, Clone)]
pub struct AsUndirected<G>(pub G);

/// Wrapper type for walking the graph as if all edges are reversed.
#[derive(Copy, Clone)]
pub struct Reversed<G>(pub G);

impl<'a, 'b, N, E: 'a, Ty, Ix> NeighborIter<'a> for AsUndirected<&'b Graph<N, E, Ty, Ix>> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Iter = graph::Neighbors<'a, E, Ix>;

    fn neighbors(&'a self, n: graph::NodeIndex<Ix>) -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors_undirected(self.0, n)
    }
}

impl<'b, N, E, Ty, Ix> IntoNeighbors for AsUndirected<&'b Graph<N, E, Ty, Ix>> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Neighbors = graph::Neighbors<'b, E, Ix>;

    fn neighbors(self, n: graph::NodeIndex<Ix>) -> graph::Neighbors<'b, E, Ix>
    {
        Graph::neighbors_undirected(self.0, n)
    }
}

impl<'a, 'b, N, E: 'a, Ty, Ix> NeighborIter<'a> for Reversed<&'b Graph<N, E, Ty, Ix>> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Iter = graph::Neighbors<'a, E, Ix>;
    fn neighbors(&'a self, n: graph::NodeIndex<Ix>) -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors_directed(self.0, n, EdgeDirection::Incoming)
    }
}

/// **Deprecated**
///
/// NeighborsDirected gives access to neighbors of both `Incoming` and `Outgoing`
/// edges of a node.
pub trait NeighborsDirected<'a> : GraphBase {
    type NeighborsDirected: Iterator<Item=Self::NodeId>;

    /// Return an iterator that visits all neighbors of the node **n**.
    fn neighbors_directed(&'a self, n: Self::NodeId,
                          d: EdgeDirection) -> Self::NeighborsDirected;
}

pub trait IntoNeighbors : GraphRef {
    type Neighbors: Iterator<Item=Self::NodeId>;
    fn neighbors(self, n: Self::NodeId) -> Self::Neighbors;
}

pub trait IntoNeighborsDirected : IntoNeighbors {
    type NeighborsDirected: Iterator<Item=Self::NodeId>;
    fn neighbors_directed(self, n: Self::NodeId, d: EdgeDirection)
        -> Self::NeighborsDirected;
}

impl<'a, N, E: 'a, Ty, Ix> IntoNeighbors for &'a Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Neighbors = graph::Neighbors<'a, E, Ix>;
    fn neighbors(self, n: graph::NodeIndex<Ix>)
        -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors(self, n)
    }
}

impl<'a, N, E: 'a, Ty, Ix> IntoNeighborsDirected for &'a Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type NeighborsDirected = graph::Neighbors<'a, E, Ix>;
    fn neighbors_directed(self, n: graph::NodeIndex<Ix>, d: EdgeDirection)
        -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors_directed(self, n, d)
    }
}

pub trait IntoNodeIdentifiers : GraphRef {
    type NodeIdentifiers: Iterator<Item=Self::NodeId>;
    fn node_identifiers(self) -> Self::NodeIdentifiers;
    fn node_count(&self) -> usize;
}

impl<'a, N, E: 'a, Ty, Ix> IntoNodeIdentifiers for &'a Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type NodeIdentifiers = graph::NodeIndices<Ix>;
    fn node_identifiers(self) -> graph::NodeIndices<Ix> {
        Graph::node_indices(self)
    }

    fn node_count(&self) -> usize {
        Graph::node_count(self)
    }
}

impl<'a, G> IntoNeighbors for &'a G
    where G: Copy + IntoNeighbors
{
    type Neighbors = G::Neighbors;
    fn neighbors(self, n: G::NodeId) -> G::Neighbors {
        (*self).neighbors(n)
    }
}

impl<'a, G> IntoNeighborsDirected for &'a G
    where G: Copy + IntoNeighborsDirected
{
    type NeighborsDirected = G::NeighborsDirected;
    fn neighbors_directed(self, n: G::NodeId, d: EdgeDirection)
        -> G::NeighborsDirected
    {
        (*self).neighbors_directed(n, d)
    }
}

impl<G> IntoNeighbors for Reversed<G>
    where G: IntoNeighborsDirected
{
    type Neighbors = G::NeighborsDirected;
    fn neighbors(self, n: G::NodeId) -> G::NeighborsDirected
    {
        self.0.neighbors_directed(n, Incoming)
    }
}

impl<G> IntoNeighborsDirected for Reversed<G>
    where G: IntoNeighborsDirected
{
    type NeighborsDirected = G::NeighborsDirected;
    fn neighbors_directed(self, n: G::NodeId, d: EdgeDirection)
        -> G::NeighborsDirected
    {
        self.0.neighbors_directed(n, d.opposite())
    }
}

impl<'a, N, E: 'a, Ty, Ix> NeighborsDirected<'a> for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type NeighborsDirected = graph::Neighbors<'a, E, Ix>;
    fn neighbors_directed(&'a self, n: graph::NodeIndex<Ix>,
                          d: EdgeDirection) -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors_directed(self, n, d)
    }
}

#[cfg(feature = "stable_graph")]
impl<'a, N, E: 'a, Ty, Ix> NeighborsDirected<'a> for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type NeighborsDirected = graph::stable::Neighbors<'a, E, Ix>;
    fn neighbors_directed(&'a self, n: graph::NodeIndex<Ix>, d: EdgeDirection)
        -> graph::stable::Neighbors<'a, E, Ix>
    {
        StableGraph::neighbors_directed(self, n, d)
    }
}

impl<'a, 'b,  G> NeighborsDirected<'a> for Reversed<&'b G>
    where G: NeighborsDirected<'a>,
{
    type NeighborsDirected = <G as NeighborsDirected<'a>>::NeighborsDirected;
    fn neighbors_directed(&'a self, n: G::NodeId,
                          d: EdgeDirection) -> Self::NeighborsDirected
    {
        self.0.neighbors_directed(n, d.opposite())
    }
}

pub trait IntoExternals : GraphRef {
    type Externals: Iterator<Item=Self::NodeId>;

    /// Return an iterator of all nodes with no edges in the given direction
    fn externals(self, d: EdgeDirection) -> Self::Externals;
}

impl<'a, N: 'a, E, Ty, Ix> IntoExternals for &'a Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Externals = graph::Externals<'a, N, Ty, Ix>;
    fn externals(self, d: EdgeDirection) -> graph::Externals<'a, N, Ty, Ix> {
        Graph::externals(self, d)
    }
}

impl<G> IntoExternals for Reversed<G>
    where G: IntoExternals,
{
    type Externals = G::Externals;
    fn externals(self, d: EdgeDirection) -> G::Externals {
        self.0.externals(d.opposite())
    }
}

/// **Deprecated**
///
/// Externals returns an iterator of all nodes that either have either no
/// incoming or no outgoing edges.
pub trait Externals<'a> : GraphBase {
    type Externals: Iterator<Item=Self::NodeId>;

    /// Return an iterator of all nodes with no edges in the given direction
    fn externals(&'a self, d: EdgeDirection) -> Self::Externals;
}

impl<'a, N: 'a, E, Ty, Ix> Externals<'a> for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Externals = graph::Externals<'a, N, Ty, Ix>;
    fn externals(&'a self, d: EdgeDirection) -> graph::Externals<'a, N, Ty, Ix> {
        Graph::externals(self, d)
    }
}

impl<'a, 'b,  G> Externals<'a> for Reversed<&'b G>
    where G: Externals<'a>,
{
    type Externals = <G as Externals<'a>>::Externals;
    fn externals(&'a self, d: EdgeDirection) -> Self::Externals {
        self.0.externals(d.opposite())
    }
}

pub trait IntoEdgeIdentifiers : GraphRef {
    type EdgeIdentifiers: Iterator<Item=Self::EdgeId>;
    fn edge_identifiers(self) -> Self::EdgeIdentifiers;
}

pub trait NodeIndexable : GraphBase {
    fn node_bound(&self) -> usize;
    fn to_index(Self::NodeId) -> usize;
}

pub trait NodeCompactIndexable : NodeIndexable { }

impl<'a, N, E, Ty, Ix> IntoEdgeIdentifiers for &'a Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type EdgeIdentifiers = graph::EdgeIndices<Ix>;
    fn edge_identifiers(self) -> Self::EdgeIdentifiers {
        self.edge_indices()
    }
}

impl<'a, G> NodeIndexable for &'a G
    where G: NodeIndexable
{
    fn node_bound(&self) -> usize { (**self).node_bound() }
    fn to_index(ix: Self::NodeId) -> usize { G::to_index(ix) }
}

impl<'a, G> NodeCompactIndexable for &'a G
    where G: NodeCompactIndexable
{ }

impl<N, E, Ty, Ix> NodeIndexable for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn node_bound(&self) -> usize { self.node_count() }
    fn to_index(ix: NodeIndex<Ix>) -> usize { ix.index() }
}
impl<N, E, Ty, Ix> NodeCompactIndexable for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{ }

/// A mapping for storing the visited status for NodeId `N`.
pub trait VisitMap<N> {
    /// Return **true** if the value is not already present.
    fn visit(&mut self, N) -> bool;
    fn is_visited(&self, &N) -> bool;
}

impl<Ix> VisitMap<graph::NodeIndex<Ix>> for FixedBitSet where
    Ix: IndexType,
{
    fn visit(&mut self, x: graph::NodeIndex<Ix>) -> bool {
        let present = self.contains(x.index());
        self.insert(x.index());
        !present
    }
    fn is_visited(&self, x: &graph::NodeIndex<Ix>) -> bool {
        self.contains(x.index())
    }
}

impl<Ix> VisitMap<graph::EdgeIndex<Ix>> for FixedBitSet where
    Ix: IndexType,
{
    fn visit(&mut self, x: graph::EdgeIndex<Ix>) -> bool {
        let present = self.contains(x.index());
        self.insert(x.index());
        !present
    }
    fn is_visited(&self, x: &graph::EdgeIndex<Ix>) -> bool {
        self.contains(x.index())
    }
}

impl<N: Eq + Hash> VisitMap<N> for HashSet<N> {
    fn visit(&mut self, x: N) -> bool {
        self.insert(x)
    }
    fn is_visited(&self, x: &N) -> bool {
        self.contains(x)
    }
}

/// A graph that can create a visitor map.
pub trait Visitable : GraphBase {
    type Map: VisitMap<Self::NodeId>;
    fn visit_map(&self) -> Self::Map;
}

/// A graph that can reset and resize its visitor map.
pub trait Revisitable : Visitable {
    fn reset_map(&self, &mut Self::Map);
}

impl<N, E, Ty, Ix> GraphBase for Graph<N, E, Ty, Ix> where
    Ix: IndexType,
{
    type NodeId = graph::NodeIndex<Ix>;
    type EdgeId = graph::EdgeIndex<Ix>;
}

impl<'a, G> Visitable for &'a G where G: Visitable {
    type Map = G::Map;
    fn visit_map(&self) -> Self::Map { (**self).visit_map() }
}

impl<N, E, Ty, Ix> Visitable for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Map = FixedBitSet;
    fn visit_map(&self) -> FixedBitSet { FixedBitSet::with_capacity(self.node_count()) }
}

impl<'a, G> Revisitable for &'a G where G: Revisitable {
    fn reset_map(&self, map: &mut Self::Map) {
        (**self).reset_map(map)
    }
}

impl<N, E, Ty, Ix> Revisitable for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
        map.grow(self.node_count());
    }
}

#[cfg(feature = "stable_graph")]
impl<N, E, Ty, Ix> GraphBase for StableGraph<N, E, Ty, Ix> where
    Ix: IndexType,
{
    type NodeId = graph::NodeIndex<Ix>;
    type EdgeId = graph::EdgeIndex<Ix>;
}

#[cfg(feature = "stable_graph")]
impl<N, E, Ty, Ix> Visitable for StableGraph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Map = FixedBitSet;
    fn visit_map(&self) -> FixedBitSet { FixedBitSet::with_capacity(self.node_count()) }
}

#[cfg(feature = "stable_graph")]
impl<N, E, Ty, Ix> Revisitable for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
        map.grow(self.node_count());
    }
}

impl<G> Revisitable for Reversed<G>
    where G: Revisitable
{
    fn reset_map(&self, map: &mut Self::Map) {
        self.0.reset_map(map);
    }
}

impl<N: Copy, E> GraphBase for GraphMap<N, E>
{
    type NodeId = N;
    type EdgeId = (N, N);
}

impl<N, E> Visitable for GraphMap<N, E>
    where N: Copy + Ord + Hash
{
    type Map = HashSet<N>;
    fn visit_map(&self) -> HashSet<N> { HashSet::with_capacity(self.node_count()) }
}

impl<N, E> Revisitable for GraphMap<N, E>
    where N: Copy + Ord + Hash
{
    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
    }
}

impl<G: GraphBase> GraphBase for AsUndirected<G>
{
    type NodeId = G::NodeId;
    type EdgeId = G::EdgeId;
}

impl<G: GraphRef> GraphRef for AsUndirected<G> { }


impl<G: Visitable> Visitable for AsUndirected<G>
{
    type Map = G::Map;
    fn visit_map(&self) -> G::Map {
        self.0.visit_map()
    }
}

impl<G: Visitable> Visitable for Reversed<G>
{
    type Map = G::Map;
    fn visit_map(&self) -> G::Map {
        self.0.visit_map()
    }
}

/// Create or access the adjacency matrix of a graph
pub trait GetAdjacencyMatrix : GraphBase {
    type AdjMatrix;
    fn adjacency_matrix(&self) -> Self::AdjMatrix;
    fn is_adjacent(&self, matrix: &Self::AdjMatrix, a: Self::NodeId, b: Self::NodeId) -> bool;
}

/// The **GraphMap** keeps an adjacency matrix internally.
impl<N, E> GetAdjacencyMatrix for GraphMap<N, E>
    where N: Copy + Ord + Hash
{
    type AdjMatrix = ();
    #[inline]
    fn adjacency_matrix(&self) { }
    #[inline]
    fn is_adjacent(&self, _: &(), a: N, b: N) -> bool {
        self.contains_edge(a, b)
    }
}

/// A depth first search (DFS) of a graph.
///
/// Using a **Dfs** you can run a traversal over a graph while still retaining
/// mutable access to it, if you use it like the following example:
///
/// ```
/// use petgraph::{Graph, Dfs};
///
/// let mut graph = Graph::<_,()>::new();
/// let a = graph.add_node(0);
///
/// let mut dfs = Dfs::new(&graph, a);
/// while let Some(nx) = dfs.next(&graph) {
///     // we can access `graph` mutably here still
///     graph[nx] += 1;
/// }
///
/// assert_eq!(graph[a], 1);
/// ```
///
/// **Note:** The algorithm may not behave correctly if nodes are removed
/// during iteration. It may not necessarily visit added nodes or edges.
#[derive(Clone, Debug)]
pub struct Dfs<N, VM> {
    /// The stack of nodes to visit
    pub stack: Vec<N>,
    /// The map of discovered nodes
    pub discovered: VM,
}

impl<N, VM> Dfs<N, VM>
    where N: Copy,
          VM: VisitMap<N>,
{
    /// Create a new **Dfs**, using the graph's visitor map, and put **start**
    /// in the stack of nodes to visit.
    pub fn new<G>(graph: G, start: N) -> Self
        where G: GraphRef + Visitable<NodeId=N, Map=VM>
    {
        let mut dfs = Dfs::empty(graph);
        dfs.move_to(start);
        dfs
    }

    /// Create a new **Dfs** using the graph's visitor map, and no stack.
    pub fn empty<G>(graph: G) -> Self
        where G: GraphRef + Visitable<NodeId=N, Map=VM>
    {
        Dfs {
            stack: Vec::new(),
            discovered: graph.visit_map(),
        }
    }

    /// Keep the discovered map, but clear the visit stack and restart
    /// the dfs from a particular node.
    pub fn move_to(&mut self, start: N)
    {
        self.discovered.visit(start.clone());
        self.stack.clear();
        self.stack.push(start);
    }

    /// Return the next node in the dfs, or **None** if the traversal is done.
    pub fn next<G>(&mut self, graph: G) -> Option<N> where
        G: IntoNeighbors<NodeId=N>,
    {
        while let Some(node) = self.stack.pop() {
            for succ in graph.neighbors(node.clone()) {
                if self.discovered.visit(succ.clone()) {
                    self.stack.push(succ);
                }
            }

            return Some(node);
        }
        None
    }
}

/// An iterator for a depth first traversal of a graph.
pub struct DfsIter<G>
    where G: GraphRef + Visitable,
{
    graph: G,
    dfs: Dfs<G::NodeId, G::Map>,
}

impl<G> DfsIter<G>
    where G: GraphRef + Visitable
{
    pub fn new(graph: G, start: G::NodeId) -> Self
    {
        // Inline the code from Dfs::new to
        // work around rust bug #22841
        let mut dfs = Dfs::empty(graph);
        dfs.move_to(start);
        DfsIter {
            graph: graph,
            dfs: dfs,
        }
    }

    /// Keep the discovered map, but clear the visit stack and restart
    /// the DFS traversal from a particular node.
    pub fn move_to(&mut self, start: G::NodeId)
    {
        self.dfs.move_to(start)
    }
}

impl<G> Iterator for DfsIter<G>
    where G: GraphRef + Visitable + IntoNeighbors
{
    type Item = G::NodeId;

    #[inline]
    fn next(&mut self) -> Option<G::NodeId>
    {
        self.dfs.next(self.graph)
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        // Very vauge info about size of traversal
        (self.dfs.stack.len(), None)
    }
}

impl<G> Clone for DfsIter<G>
    where G: GraphRef + Visitable,
          Dfs<G::NodeId, G::Map>: Clone
{
    fn clone(&self) -> Self {
        DfsIter {
            graph: self.graph,
            dfs: self.dfs.clone(),
        }
    }
}

/// A breadth first search (BFS) of a graph.
///
/// Using a **Bfs** you can run a traversal over a graph while still retaining
/// mutable access to it, if you use it like the following example:
///
/// ```
/// use petgraph::{Graph, Bfs};
///
/// let mut graph = Graph::<_,()>::new();
/// let a = graph.add_node(0);
///
/// let mut bfs = Bfs::new(&graph, a);
/// while let Some(nx) = bfs.next(&graph) {
///     // we can access `graph` mutably here still
///     graph[nx] += 1;
/// }
///
/// assert_eq!(graph[a], 1);
/// ```
///
/// **Note:** The algorithm may not behave correctly if nodes are removed
/// during iteration. It may not necessarily visit added nodes or edges.
#[derive(Clone)]
pub struct Bfs<N, VM> {
    /// The queue of nodes to visit
    pub stack: VecDeque<N>,
    /// The map of discovered nodes
    pub discovered: VM,
}

impl<N, VM> Bfs<N, VM>
    where N: Copy,
          VM: VisitMap<N>,
{
    /// Create a new **Bfs**, using the graph's visitor map, and put **start**
    /// in the stack of nodes to visit.
    pub fn new<G>(graph: &G, start: N) -> Self
        where G: Visitable<NodeId=N, Map=VM>
    {
        let mut discovered = graph.visit_map();
        discovered.visit(start.clone());
        let mut stack = VecDeque::new();
        stack.push_front(start.clone());
        Bfs {
            stack: stack,
            discovered: discovered,
        }
    }

    /// Return the next node in the dfs, or **None** if the traversal is done.
    pub fn next<G>(&mut self, graph: G) -> Option<N> where
        G: IntoNeighbors<NodeId=N>
    {
        while let Some(node) = self.stack.pop_front() {
            for succ in graph.neighbors(node.clone()) {
                if self.discovered.visit(succ.clone()) {
                    self.stack.push_back(succ);
                }
            }

            return Some(node);
        }
        None
    }

}

/// An iterator for a breadth first traversal of a graph.
pub struct BfsIter<G: Visitable> {
    graph: G,
    bfs: Bfs<G::NodeId, G::Map>,
}

impl<G: Visitable> BfsIter<G>
    where G::NodeId: Copy,
          G: GraphRef,
{
    pub fn new(graph: G, start: G::NodeId) -> Self
    {
        // Inline the code from Bfs::new to
        // work around rust bug #22841
        let mut discovered = graph.visit_map();
        discovered.visit(start.clone());
        let mut stack = VecDeque::new();
        stack.push_front(start.clone());
        let bfs = Bfs {
            stack: stack,
            discovered: discovered,
        };
        BfsIter {
            graph: graph,
            bfs: bfs,
        }
    }
}

impl< G: Visitable> Iterator for BfsIter<G>
    where G: IntoNeighbors,
{
    type Item = G::NodeId;
    fn next(&mut self) -> Option<G::NodeId> {
        self.bfs.next(self.graph)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.bfs.stack.len(), None)
    }
}

impl<G: Visitable> Clone for BfsIter<G>
    where Bfs<G::NodeId, G::Map>: Clone,
          G: GraphRef
{
    fn clone(&self) -> Self {
        BfsIter {
            graph: self.graph,
            bfs: self.bfs.clone(),
        }
    }
}


/// A topological order traversal for a graph.
#[derive(Clone)]
pub struct Topo<N, VM> {
    tovisit: Vec<N>,
    ordered: VM,
}

impl<N, VM> Topo<N, VM>
    where N: Copy,
          VM: VisitMap<N>,
{
    /// Create a new **Topo**, using the graph's visitor map, and put all
    /// initial nodes in the to visit list.
    pub fn new<G>(graph: G) -> Self
        where G: IntoExternals + Visitable<NodeId=N, Map=VM>,
    {
        let mut topo = Self::empty(graph);
        topo.tovisit.extend(graph.externals(Incoming));
        topo
    }

    /* Private ntil it has a use */
    /// Create a new **Topo**, using the graph's visitor map with *no* starting
    /// index specified.
    fn empty<G>(graph: G) -> Self
        where G: GraphRef + Visitable<NodeId=N, Map=VM>
    {
        Topo {
            ordered: graph.visit_map(),
            tovisit: Vec::new(),
        }
    }

    /// Clear visited state, and put all initial nodes in the to visit list.
    pub fn reset<G>(&mut self, graph: G)
        where G: IntoExternals + Revisitable<NodeId=N, Map=VM>,
    {
        graph.reset_map(&mut self.ordered);
        self.tovisit.clear();
        self.tovisit.extend(graph.externals(Incoming));
    }

    /// Return the next node in the current topological order traversal, or
    /// `None` if the traversal is at end.
    ///
    /// *Note:* The graph may not have a complete topological order, and the only
    /// way to know is to run the whole traversal and make sure it visits every node.
    pub fn next<G>(&mut self, g: G) -> Option<N>
        where G: IntoNeighborsDirected + Visitable<NodeId=N, Map=VM>,
    {
        // Take an unvisited element and find which of its neighbors are next
        while let Some(nix) = self.tovisit.pop() {
            if self.ordered.is_visited(&nix) {
                continue;
            }
            self.ordered.visit(nix.clone());
            for neigh in g.neighbors(nix) {
                // Look at each neighbor, and those that only have incoming edges
                // from the already ordered list, they are the next to visit.
                if Reversed(g).neighbors(neigh).all(|b| self.ordered.is_visited(&b)) {
                    self.tovisit.push(neigh);
                }
            }
            return Some(nix);
        }
        None
    }
}

