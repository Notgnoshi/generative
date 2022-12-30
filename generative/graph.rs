use geo::Point;

pub type NodeData = Point;
pub type EdgeWeight = ();
pub type NodeIndex = usize;

pub type GeometryGraph<Direction> = petgraph::Graph<NodeData, EdgeWeight, Direction, NodeIndex>;
