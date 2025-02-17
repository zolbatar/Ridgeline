use crate::geo::data::WayPoint;
use geo::{Coord, LineString, MultiLineString, Simplify};
use ordered_float::NotNan;
use petgraph::graphmap::UnGraphMap;
use std::collections::{HashSet, VecDeque};

/// Convert WayPoints into a Graph (Fix: Use NotNan<f64>)
fn build_graph(waypoints: Vec<WayPoint>) -> UnGraphMap<(NotNan<f64>, NotNan<f64>), ()> {
    let mut graph = UnGraphMap::new();

    for i in 0..waypoints.len() - 1 {
        let wp1 = &waypoints[i];
        let wp2 = &waypoints[i + 1];

        let node1 = (NotNan::new(wp1.x).unwrap(), NotNan::new(wp1.y).unwrap());
        let node2 = (NotNan::new(wp2.x).unwrap(), NotNan::new(wp2.y).unwrap());

        graph.add_node(node1);
        graph.add_node(node2);

        // Only connect if `is_start` is not enforced as a separate move
        if !wp2.is_start {
            graph.add_edge(node1, node2, ());
        }
    }

    graph
}

/// Extract paths using DFS traversal while ensuring "Move-To" (`is_start = true`) is respected
fn extract_paths(graph: &UnGraphMap<(NotNan<f64>, NotNan<f64>), ()>, waypoints: &Vec<WayPoint>) -> Vec<LineString<f64>> {
    let mut visited = HashSet::new();
    let mut paths = Vec::new();

    // Find `is_start` nodes explicitly to identify disconnected components
    let start_nodes: HashSet<_> =
        waypoints.iter().filter(|wp| wp.is_start).map(|wp| (NotNan::new(wp.x).unwrap(), NotNan::new(wp.y).unwrap())).collect();

    for start in graph.nodes() {
        if visited.contains(&start) {
            continue;
        }

        let mut stack = VecDeque::new();
        let mut path_coords = Vec::new();

        stack.push_back(start);
        let mut first = true; // Track first point in the component

        while let Some(node) = stack.pop_back() {
            if !visited.insert(node) {
                continue;
            }

            // Ensure move points (`is_start = true`) are properly marked at the start of disjoint paths
            if first || start_nodes.contains(&node) {
                path_coords.push(Coord {
                    x: node.0.into_inner(),
                    y: node.1.into_inner(),
                });
                first = false;
            }

            for neighbor in graph.neighbors(node) {
                stack.push_back(neighbor);
            }
        }

        if path_coords.len() > 1 {
            paths.push(LineString::from(path_coords));
        }
    }

    paths
}

/// Convert unordered WayPoints into a `MultiLineString` with move-to points preserved
pub fn waypoints_to_multilinestring(waypoints: Vec<WayPoint>) -> MultiLineString<f64> {
    let graph = build_graph(waypoints.clone());
    MultiLineString(extract_paths(&graph, &waypoints))
}

/// Convert `MultiLineString<f64>` back into a `Vec<WayPoint>`
pub fn multilinestring_to_waypoints(multi_line: MultiLineString<f64>) -> Vec<WayPoint> {
    let mut waypoints = Vec::new();

    for line in multi_line.0 {
        let mut first = true; // Track first point in each line

        for coord in line.0 {
            waypoints.push(WayPoint {
                is_start: first, // First point in each LineString gets `is_start = true`
                x: coord.x,
                y: coord.y,
            });

            first = false;
        }
    }

    waypoints
}

/// Simplify a MultiLineString using Douglas-Peucker algorithm
pub fn simplify_multilinestring(multi_line: &MultiLineString<f64>, tolerance: f64) -> MultiLineString<f64> {
    multi_line.simplify(&tolerance)
}
