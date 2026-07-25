use crate::comparison::Counts;
use crate::report::Report;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Recorded accuracy numbers, checked into the repository so that any change in detection
/// behaviour shows up as a reviewable diff.
///
/// Only integer counts are stored; precision and recall are derived when reporting, so the
/// file does not churn on float formatting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Baseline {
    pub fixtures: BTreeMap<String, Counts>,
    pub total: Counts,
}

impl Baseline {
    pub fn from_report(report: &Report) -> Self {
        Self {
            fixtures: report
                .fixtures
                .iter()
                .map(|fixture| (fixture.name.clone(), fixture.counts.clone()))
                .collect(),
            total: report.totals(),
        }
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;

        serde_json::from_str(&content)
            .map_err(|error| format!("failed to parse {}: {error}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|error| format!("failed to serialize baseline: {error}"))?;

        fs::write(path, format!("{content}\n"))
            .map_err(|error| format!("failed to write {}: {error}", path.display()))
    }

    /// Per-fixture differences against another baseline, as human-readable lines.
    pub fn diff(&self, other: &Baseline) -> Vec<String> {
        self.fixture_names(other)
            .into_iter()
            .filter_map(|name| self.diff_fixture(other, &name))
            .collect()
    }

    fn fixture_names(&self, other: &Baseline) -> Vec<String> {
        let mut names: Vec<String> = self
            .fixtures
            .keys()
            .chain(other.fixtures.keys())
            .cloned()
            .collect();
        names.sort();
        names.dedup();
        names
    }

    fn diff_fixture(&self, other: &Baseline, name: &str) -> Option<String> {
        let recorded = self.fixtures.get(name);
        let current = other.fixtures.get(name);

        match (recorded, current) {
            (Some(recorded), Some(current)) if recorded != current => Some(format!(
                "{name}\n  recorded: {recorded:?}\n  current:  {current:?}"
            )),
            (Some(_), Some(_)) => None,
            (Some(_), None) => Some(format!(
                "{name}\n  fixture is in the baseline but was not found"
            )),
            (None, Some(current)) => Some(format!(
                "{name}\n  new fixture, not in the baseline\n  current: {current:?}"
            )),
            (None, None) => None,
        }
    }
}
