use crate::analysis::detect;
use crate::comparison::{compare, Counts, FixtureReport};
use crate::expectation::parse_expectations;
use crate::fixtures::{discover, fixture_name};
use comfy_table::{presets::UTF8_FULL, Table};
use std::fs;
use std::path::Path;

/// Accuracy of dependency detection across every fixture.
pub struct Report {
    pub fixtures: Vec<FixtureReport>,
}

impl Report {
    /// Analyze every fixture under `root` and compare against its annotations.
    pub fn run(root: &Path) -> Result<Self, String> {
        let paths = discover(root)
            .map_err(|error| format!("failed to read fixtures in {}: {error}", root.display()))?;

        let fixtures = paths
            .iter()
            .map(|path| run_fixture(root, path))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { fixtures })
    }

    pub fn totals(&self) -> Counts {
        self.fixtures.iter().map(|fixture| &fixture.counts).sum()
    }

    /// Per-fixture counts plus an aggregate row.
    pub fn to_table(&self) -> String {
        let mut table = Table::new();
        table.load_style(UTF8_FULL);
        table.set_header(vec![
            "Fixture",
            "Expected",
            "Detected",
            "Correct",
            "Missing",
            "Spurious",
            "Duplicates",
            "Precision",
            "Recall",
        ]);

        self.fixtures.iter().for_each(|fixture| {
            table.add_row(row(&fixture.name, &fixture.counts));
        });

        table.add_row(row("TOTAL", &self.totals()));
        table.to_string()
    }

    /// The individual missing and spurious edges, for diagnosing where accuracy is lost.
    pub fn details(&self) -> String {
        self.fixtures
            .iter()
            .filter(|fixture| !fixture.missing.is_empty() || !fixture.spurious.is_empty())
            .map(fixture_details)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn run_fixture(root: &Path, path: &Path) -> Result<FixtureReport, String> {
    let name = fixture_name(root, path);
    let source = fs::read_to_string(path)
        .map_err(|error| format!("{name}: failed to read fixture: {error}"))?;
    let expected = parse_expectations(&source).map_err(|error| format!("{name}: {error}"))?;
    let detected = detect(path).map_err(|error| format!("{name}: analysis failed: {error}"))?;

    Ok(compare(name, &expected, detected))
}

fn row(name: &str, counts: &Counts) -> Vec<String> {
    vec![
        name.to_string(),
        counts.expected.to_string(),
        counts.detected.to_string(),
        counts.correct.to_string(),
        counts.missing.to_string(),
        counts.spurious.to_string(),
        counts.duplicates.to_string(),
        format!("{:.3}", counts.precision()),
        format!("{:.3}", counts.recall()),
    ]
}

fn fixture_details(fixture: &FixtureReport) -> String {
    let missing = edge_lines("missing", &fixture.missing);
    let spurious = edge_lines("spurious", &fixture.spurious);
    let shift = crate::shift::detect(&fixture.missing, &fixture.spurious)
        .map(crate::shift::explain)
        .unwrap_or_default();

    format!("{}\n{missing}{spurious}{shift}", fixture.name)
}

fn edge_lines(label: &str, edges: &[crate::edge::Edge]) -> String {
    edges
        .iter()
        .map(|edge| format!("  {label}: {edge}\n"))
        .collect()
}
