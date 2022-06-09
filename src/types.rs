use crate::io::wtdbg2::{PlainWtdbg2EdgeData, PlainWtdbg2NodeData};

/// A node-centric genome graph with `PlainBCalm2NodeData` as node data represented using the `petgraph` crate.
pub type PetBCalm2NodeGraph<GenomeSequenceStoreHandle> =
    crate::bigraph::implementation::node_bigraph_wrapper::NodeBigraphWrapper<
        crate::bigraph::traitgraph::implementation::petgraph_impl::PetGraph<
            crate::io::bcalm2::PlainBCalm2NodeData<GenomeSequenceStoreHandle>,
            (),
        >,
    >;

/// An edge-centric genome graph with `PlainBCalm2NodeData` as edge data represented using the `petgraph` crate.
pub type PetBCalm2EdgeGraph<GenomeSequenceStoreHandle> =
    crate::bigraph::implementation::node_bigraph_wrapper::NodeBigraphWrapper<
        crate::bigraph::traitgraph::implementation::petgraph_impl::PetGraph<
            (),
            crate::io::bcalm2::PlainBCalm2NodeData<GenomeSequenceStoreHandle>,
        >,
    >;

/// A genome graph for the wtdbg2 assembler represented using the `petgraph` crate.
pub type PetWtdbg2Graph = crate::bigraph::implementation::node_bigraph_wrapper::NodeBigraphWrapper<
    crate::bigraph::traitgraph::implementation::petgraph_impl::PetGraph<
        PlainWtdbg2NodeData,
        PlainWtdbg2EdgeData,
    >,
>;

/// Simple type to represent bigraphs from the .dot format.
pub type PetWtdbg2DotGraph =
    crate::bigraph::implementation::node_bigraph_wrapper::NodeBigraphWrapper<
        crate::bigraph::traitgraph::implementation::petgraph_impl::PetGraph<String, ()>,
    >;
