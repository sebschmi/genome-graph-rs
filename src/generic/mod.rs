use bigraph::interface::dynamic_bigraph::DynamicEdgeCentricBigraph;
use bigraph::interface::BidirectedData;
use bigraph::traitgraph::index::GraphIndex;
use bigraph::traitgraph::interface::GraphBase;
use std::fmt::Formatter;

pub(crate) enum MappedNode<Graph: GraphBase> {
    Unmapped,
    Normal {
        forward: Graph::NodeIndex,
        backward: Graph::NodeIndex,
    },
    SelfMirror(Graph::NodeIndex),
}

impl<Graph: GraphBase> MappedNode<Graph> {
    pub(crate) fn mirror(self) -> Self {
        match self {
            MappedNode::Unmapped => MappedNode::Unmapped,
            MappedNode::Normal { forward, backward } => MappedNode::Normal {
                forward: backward,
                backward: forward,
            },
            MappedNode::SelfMirror(node) => MappedNode::SelfMirror(node),
        }
    }
}

impl<Graph: GraphBase> std::fmt::Debug for MappedNode<Graph> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MappedNode::Unmapped => write!(f, "unmapped"),
            MappedNode::Normal { forward, backward } => {
                write!(f, "({}, {})", forward.as_usize(), backward.as_usize())
            }
            MappedNode::SelfMirror(node) => write!(f, "{}", node.as_usize()),
        }
    }
}

impl<Graph: GraphBase> Clone for MappedNode<Graph> {
    fn clone(&self) -> Self {
        match self {
            MappedNode::Unmapped => MappedNode::Unmapped,
            MappedNode::Normal { forward, backward } => MappedNode::Normal {
                forward: *forward,
                backward: *backward,
            },
            MappedNode::SelfMirror(node) => MappedNode::SelfMirror(*node),
        }
    }
}

impl<Graph: GraphBase> Copy for MappedNode<Graph> {}

impl<Graph: GraphBase> PartialEq for MappedNode<Graph> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MappedNode::Unmapped, MappedNode::Unmapped) => true,
            (
                MappedNode::Normal {
                    forward: f1,
                    backward: b1,
                },
                MappedNode::Normal {
                    forward: f2,
                    backward: b2,
                },
            ) => f1 == f2 && b1 == b2,
            (MappedNode::SelfMirror(n1), MappedNode::SelfMirror(n2)) => n1 == n2,
            _ => false,
        }
    }
}

impl<Graph: GraphBase> Eq for MappedNode<Graph> {}

/// A node representing a unitig with the edges of a bidirected de Bruijn graph, inspired by bcalm2's fasta format.
pub trait GenericNode {
    /// The iterator used to iterate over this node's edges.
    type EdgeIterator: Iterator<Item = GenericEdge>;

    /// A unique identifier of this node.
    /// The identifiers of the nodes need to be numbered consecutively starting from 0.
    fn id(&self) -> usize;

    /// Return true if this node is self-complemental, i.e. if the reverse complement of the first k-1 characters equals the last k-1 characters.
    fn is_self_complemental(&self) -> bool;

    /// Return an iterator over the edges of the node.
    /// It is enough to return the edges that also bcalm2 would return.
    fn edges(&self) -> Self::EdgeIterator;
}

/// An edge representing a k-1 overlap between unitigs.
///
/// Terminology: the edge goes from "tail" to "head".
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct GenericEdge {
    /// The direction of the unitig at the tail of the edge.
    pub from_side: bool,
    /// The id of the unitig at the head of the edge.
    pub to_node: usize,
    /// The direction of the unitig at the head of the edge.
    pub to_side: bool,
}

/// Read a genome graph in bcalm2 fasta format into an edge-centric representation.
pub fn convert_generic_node_centric_bigraph_to_edge_centric<
    GenomeSequenceStoreHandle,
    NodeData: Default + Clone,
    InputEdgeData: GenericNode,
    OutputEdgeData: From<InputEdgeData> + Clone + Eq + BidirectedData,
    Graph: DynamicEdgeCentricBigraph<NodeData = NodeData, EdgeData = OutputEdgeData> + Default,
>(
    reader: impl IntoIterator<Item = InputEdgeData>,
) -> crate::error::Result<Graph>
where
    <Graph as GraphBase>::NodeIndex: Clone,
{
    let mut node_map: Vec<MappedNode<Graph>> = Vec::new();
    let mut graph = Graph::default();

    for generic_node in reader.into_iter() {
        let edge_is_self_mirror = generic_node.is_self_complemental();

        let n1 = generic_node.id() * 2;
        let n2 = generic_node.id() * 2 + 1;

        let n1_is_self_mirror = generic_node.edges().any(|edge| {
            edge == GenericEdge {
                from_side: false,
                to_node: generic_node.id(),
                to_side: true,
            }
        });
        let n2_is_self_mirror = generic_node.edges().any(|edge| {
            edge == GenericEdge {
                from_side: true,
                to_node: generic_node.id(),
                to_side: false,
            }
        });

        if node_map.len() <= n2 {
            node_map.resize(n2 + 1, MappedNode::Unmapped);
        }

        // If the record has no known incoming binode yet
        if node_map[n1] == MappedNode::Unmapped {
            let mut assign_to_neighbors = false;

            // If the record has no known incoming binode yet, first search if one of the neighbors exist
            for edge in generic_node
                .edges()
                // Incoming edges to n1 are outgoing on its reverse complement
                .filter(|edge| !edge.from_side)
            {
                // Location of the to_node of the edge in the node_map
                let to_node = edge.to_node * 2 + if edge.to_side { 0 } else { 1 };

                if node_map.len() <= to_node {
                    node_map.resize(to_node + 1, MappedNode::Unmapped);
                }
                if node_map[to_node] != MappedNode::Unmapped {
                    node_map[n1] = if !edge.to_side {
                        node_map[to_node]
                    } else {
                        // If the edge changes sides, the node is mirrored
                        node_map[to_node].mirror()
                    };
                    assign_to_neighbors = true;
                    break;
                }
            }

            // If no neighbor was found, create a new binode and also assign it to the neighbors
            if node_map[n1] == MappedNode::Unmapped {
                if n1_is_self_mirror {
                    let n1s = graph.add_node(NodeData::default());
                    graph.set_mirror_nodes(n1s, n1s);
                    node_map[n1] = MappedNode::SelfMirror(n1s);
                } else {
                    let n1f = graph.add_node(NodeData::default());
                    let n1r = graph.add_node(NodeData::default());
                    graph.set_mirror_nodes(n1f, n1r);
                    node_map[n1] = MappedNode::Normal {
                        forward: n1f,
                        backward: n1r,
                    };
                }
                assign_to_neighbors = true;
            }

            if assign_to_neighbors {
                // Assign the new node also to the neighbors
                for edge in generic_node
                    .edges()
                    // Incoming edges to n1 are outgoing on its reverse complement
                    .filter(|edge| !edge.from_side)
                {
                    // Location of the to_node of the edge in the node_map
                    let to_node = edge.to_node * 2 + if edge.to_side { 0 } else { 1 };
                    node_map[to_node] = if !edge.to_side {
                        node_map[n1]
                    } else {
                        // If the edge changes sides, the node is mirrored
                        node_map[n1].mirror()
                    };
                }
            }
        }

        // If the record has no known outgoing binode yet
        if node_map[n2] == MappedNode::Unmapped {
            let mut assign_to_neighbors = false;

            if edge_is_self_mirror {
                node_map[n2] = node_map[n1].mirror();
                // not sure if needed, but should be rare enough that it is not worth to think about it (and it is correct like this as well)
                assign_to_neighbors = true;
            } else {
                // If the record has no known outgoing binode yet, first search if one of the neighbors exist
                for edge in generic_node
                    .edges()
                    // Outgoing edges from n1 are outgoing from its forward variant
                    .filter(|edge| edge.from_side)
                {
                    // Location of the to_node of the edge in the node_map
                    let to_node = edge.to_node * 2 + if edge.to_side { 0 } else { 1 };

                    if node_map.len() <= to_node {
                        node_map.resize(to_node + 1, MappedNode::Unmapped);
                    }
                    if node_map[to_node] != MappedNode::Unmapped {
                        node_map[n2] = if edge.to_side {
                            node_map[to_node]
                        } else {
                            // If the edge changes sides, the node is mirrored
                            node_map[to_node].mirror()
                        };
                        assign_to_neighbors = true;
                        break;
                    }
                }

                // If no neighbor was found, create a new binode and also assign it to the neighbors
                if node_map[n2] == MappedNode::Unmapped {
                    if n2_is_self_mirror {
                        let n2s = graph.add_node(NodeData::default());
                        graph.set_mirror_nodes(n2s, n2s);
                        node_map[n2] = MappedNode::SelfMirror(n2s);
                    } else {
                        let n2f = graph.add_node(NodeData::default());
                        let n2r = graph.add_node(NodeData::default());
                        graph.set_mirror_nodes(n2f, n2r);
                        node_map[n2] = MappedNode::Normal {
                            forward: n2f,
                            backward: n2r,
                        };
                    }
                    assign_to_neighbors = true;
                }
            }

            if assign_to_neighbors {
                // Assign the new node also to the neighbors
                for edge in generic_node
                    .edges()
                    // Outgoing edges from n1 are outgoing from its forward variant
                    .filter(|edge| edge.from_side)
                {
                    // Location of the to_node of the edge in the node_map
                    let to_node = edge.to_node * 2 + if edge.to_side { 0 } else { 1 };
                    node_map[to_node] = if edge.to_side {
                        node_map[n2]
                    } else {
                        // If the edge changes sides, the node is mirrored
                        node_map[n2].mirror()
                    };
                }
            }
        }

        debug_assert_ne!(node_map[n1], MappedNode::Unmapped);
        debug_assert_ne!(node_map[n2], MappedNode::Unmapped);

        let (n1f, n1r) = match node_map[n1] {
            MappedNode::Unmapped => unreachable!(),
            MappedNode::Normal { forward, backward } => (forward, backward),
            MappedNode::SelfMirror(node) => (node, node),
        };
        let (n2f, n2r) = match node_map[n2] {
            MappedNode::Unmapped => unreachable!(),
            MappedNode::Normal { forward, backward } => (forward, backward),
            MappedNode::SelfMirror(node) => (node, node),
        };

        let edge_data: OutputEdgeData = generic_node.into();
        graph.add_edge(n1f, n2f, edge_data.clone());
        graph.add_edge(n2r, n1r, edge_data.mirror());
    }

    Ok(graph)
}
