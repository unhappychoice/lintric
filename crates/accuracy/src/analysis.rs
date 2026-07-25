use crate::edge::Edge;
use lintric_core::models::Dependency;
use std::collections::BTreeSet;
use std::path::Path;

/// Dependencies the analyzer reported for a fixture.
pub struct Detected {
    /// Distinct edges, which is what the line-to-line dependency graph should contain.
    pub edges: BTreeSet<Edge>,
    /// Edges the analyzer emitted more than once. Tracked separately so that over-counting
    /// stays visible instead of being absorbed into the precision figure.
    pub duplicates: usize,
}

/// Run the analyzer over a fixture and collect the dependencies it reports.
pub fn detect(path: &Path) -> Result<Detected, String> {
    let ir = lintric_core::get_intermediate_representation(path.to_string_lossy().into_owned())?;
    let reported: Vec<Edge> = ir.dependencies.iter().map(to_edge).collect();
    let edges: BTreeSet<Edge> = reported.iter().cloned().collect();

    Ok(Detected {
        duplicates: reported.len() - edges.len(),
        edges,
    })
}

fn to_edge(dependency: &Dependency) -> Edge {
    Edge::new(
        dependency.source_line,
        dependency.target_line,
        dependency.symbol.clone(),
    )
}
