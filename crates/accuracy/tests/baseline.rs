use lintric_accuracy::baseline::Baseline;
use lintric_accuracy::report::Report;
use lintric_accuracy::{baseline_path, fixtures_dir};

/// Accuracy must match the recorded baseline exactly, in either direction. An improvement is
/// as much a reason to update the baseline as a regression is, so that every change in
/// detection behaviour is visible in a diff.
#[test]
fn accuracy_matches_recorded_baseline() {
    let report = Report::run(&fixtures_dir()).expect("failed to run accuracy report");
    let recorded = Baseline::load(&baseline_path()).expect("failed to load baseline");
    let differences = recorded.diff(&Baseline::from_report(&report));

    assert!(
        differences.is_empty(),
        "accuracy differs from the recorded baseline:\n\n{}\n\n{}\n\nrun `cargo run -p lintric-accuracy -- --update` to record these numbers",
        differences.join("\n"),
        report.to_table()
    );
}

/// Every fixture must contribute expectations, otherwise it silently measures nothing.
#[test]
fn every_fixture_is_annotated() {
    let report = Report::run(&fixtures_dir()).expect("failed to run accuracy report");

    let unannotated: Vec<&str> = report
        .fixtures
        .iter()
        .filter(|fixture| fixture.counts.expected == 0)
        .map(|fixture| fixture.name.as_str())
        .collect();

    assert!(
        unannotated.is_empty(),
        "fixtures without any `//~ depends:` annotation: {unannotated:?}"
    );
}
