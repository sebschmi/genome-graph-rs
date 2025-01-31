use crate::bigraph::interface::dynamic_bigraph::DynamicBigraph;
use crate::bigraph::traitgraph::traitsequence::interface::Sequence;
use crate::error::Result;
use bigraph::traitgraph::interface::StaticGraph;
use bigraph::traitgraph::walks::{EdgeWalk, VecNodeWalk};
use error::DotIoError;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;

pub mod error;

/// Node data of a dot graph node.
pub trait DotNodeData {
    /// The name of the node in the .dot file.
    fn node_name(&self) -> &str;
}

impl DotNodeData for String {
    fn node_name(&self) -> &str {
        self
    }
}

/// Read a bigraph in dot format from a file.
pub fn read_graph_from_wtdbg2_dot_from_file<
    P: AsRef<Path>,
    NodeData: FromStr + Debug,
    EdgeData: Default,
    Graph: DynamicBigraph<NodeData = NodeData, EdgeData = EdgeData> + Default,
>(
    dot_file: P,
) -> Result<Graph>
where
    <NodeData as FromStr>::Err: Debug,
{
    read_graph_from_wtdbg2_dot(BufReader::new(File::open(dot_file)?))
}

/// Read a bigraph in dot format from a `BufRead`.
pub fn read_graph_from_wtdbg2_dot<
    R: BufRead,
    NodeData: FromStr + Debug,
    EdgeData: Default,
    Graph: DynamicBigraph<NodeData = NodeData, EdgeData = EdgeData> + Default,
>(
    dot: R,
) -> Result<Graph>
where
    <NodeData as FromStr>::Err: Debug,
{
    let mut graph = Graph::default();
    let mut node_id_map = HashMap::new();

    enum State {
        KwDigraph,
        OpenBrace,
        KwNode,
        KwShapeRecord,
        Nodes,
        Edges,
        CloseBrace,
        Ok,
    }
    let mut state = State::KwDigraph;

    for line in dot.lines() {
        let line = line?;
        let mut line = line.trim();

        while !line.is_empty() {
            match state {
                State::KwDigraph => {
                    DotIoError::expect_line_start(line, "digraph")?;
                    line = line[7..].trim();
                    state = State::OpenBrace;
                }
                State::OpenBrace => {
                    DotIoError::expect_line_start(line, "{")?;
                    line = line[1..].trim();
                    state = State::KwNode;
                }
                State::KwNode => {
                    DotIoError::expect_line_start(line, "node")?;
                    line = line[4..].trim();
                    state = State::KwShapeRecord;
                }
                State::KwShapeRecord => {
                    const SHAPE_RECORD: &str = "[shape=record]";
                    DotIoError::expect_line_start(line, SHAPE_RECORD)?;
                    line = line[SHAPE_RECORD.len()..].trim();
                    DotIoError::expect_empty_line(line)?;
                    state = State::Nodes;
                }
                State::Nodes => {
                    let mut tokens = line.split(' ');
                    let node_name = tokens.next().expect("Expected line to be non-empty.");
                    if let Some(arrow) = tokens.next() {
                        if arrow == "->" {
                            state = State::Edges;
                            continue;
                        }
                    }

                    let forward_node_name = node_name.to_string() + " +";
                    let backward_node_name = node_name.to_string() + " -";
                    let forward_node_id =
                        graph.add_node(FromStr::from_str(&forward_node_name).unwrap());
                    let backward_node_id =
                        graph.add_node(FromStr::from_str(&backward_node_name).unwrap());
                    graph.set_mirror_nodes(forward_node_id, backward_node_id);

                    if node_id_map
                        .insert(forward_node_name.clone(), forward_node_id)
                        .is_some()
                    {
                        return Err(DotIoError::DuplicateNodeId {
                            name: forward_node_name,
                        }
                        .into());
                    }
                    if node_id_map
                        .insert(backward_node_name.clone(), backward_node_id)
                        .is_some()
                    {
                        return Err(DotIoError::DuplicateNodeId {
                            name: backward_node_name,
                        }
                        .into());
                    }

                    line = "";
                }
                State::Edges => {
                    let mut tokens = line.split(' ');
                    let mut from_node_name = tokens
                        .next()
                        .expect("Expected line to be non-empty.")
                        .to_string();
                    if from_node_name == "}" {
                        state = State::CloseBrace;
                        continue;
                    }

                    let arrow = tokens.next().expect("Missing arrow token in edge.");
                    DotIoError::expect_token(arrow, "->")?;
                    let mut to_node_name =
                        tokens.next().expect("Missing to node in edge.").to_string();

                    let direction_label = tokens.next().expect("Missing direction label in edge.");
                    const DIRECTION_LABEL_PREFIX: &str = "[label=\"";
                    DotIoError::expect_line_start(direction_label, DIRECTION_LABEL_PREFIX)?;
                    let direction_label = &direction_label[DIRECTION_LABEL_PREFIX.len()..][..2];
                    from_node_name += " ";
                    from_node_name += &direction_label[0..1];
                    to_node_name += " ";
                    to_node_name += &direction_label[1..2];

                    let from_node_id = node_id_map
                        .get(&from_node_name)
                        .expect("Unknown from node name in edge");
                    let to_node_id = node_id_map
                        .get(&to_node_name)
                        .expect("Unknown to node name in edge");
                    graph.add_edge(*from_node_id, *to_node_id, Default::default());
                    line = "";
                }
                State::CloseBrace => {
                    DotIoError::expect_line_start(line, "}")?;
                    line = line[1..].trim();
                    state = State::Ok;
                }
                State::Ok => {
                    return Err(DotIoError::UnexpectedToken {
                        expected: "".to_string(),
                        actual: line.to_string(),
                    }
                    .into());
                }
            }
        }
    }

    Ok(graph)
}

/// Write a list of contigs as lists of node ids to a file.
/// The ids are accompanied by a + or - indicating their direction.
pub fn write_dot_contigs_as_wtdbg2_node_ids_to_file<
    'ws,
    P: AsRef<Path>,
    NodeData: DotNodeData,
    EdgeData,
    Graph: StaticGraph<NodeData = NodeData, EdgeData = EdgeData>,
    Walk: 'ws + EdgeWalk<Graph, Subwalk>,
    Subwalk: EdgeWalk<Graph, Subwalk> + ?Sized,
    WalkSource: 'ws + IntoIterator<Item = &'ws Walk>,
>(
    graph: &Graph,
    walks: WalkSource,
    output_file: P,
) -> Result<()> {
    write_dot_contigs_as_wtdbg2_node_ids(
        graph,
        walks,
        &mut BufWriter::new(File::create(output_file)?),
    )
}

/// Write a list of contigs as lists of node ids.
/// The ids are accompanied by a + or - indicating their direction.
pub fn write_dot_contigs_as_wtdbg2_node_ids<
    'ws,
    W: Write,
    NodeData: DotNodeData,
    EdgeData,
    Graph: StaticGraph<NodeData = NodeData, EdgeData = EdgeData>,
    Walk: 'ws + EdgeWalk<Graph, Subwalk>,
    Subwalk: EdgeWalk<Graph, Subwalk> + ?Sized,
    WalkSource: 'ws + IntoIterator<Item = &'ws Walk>,
>(
    graph: &Graph,
    walks: WalkSource,
    output: &mut W,
) -> Result<()> {
    for walk in walks {
        let walk: VecNodeWalk<Graph> = walk.clone_as_node_walk(graph).unwrap();
        for &node in walk.iter() {
            write!(output, "{} ", graph.node_data(node).node_name(),)?;
        }
        writeln!(output)?;
    }

    Ok(())
}
