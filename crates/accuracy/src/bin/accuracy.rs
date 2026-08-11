use lintric_accuracy::baseline::Baseline;
use lintric_accuracy::report::Report;
use lintric_accuracy::{baseline_path, fixtures_dir};

/// Reports dependency detection accuracy against the annotated fixtures.
///
/// `accuracy`          print the report
/// `accuracy --check`  exit non-zero if the numbers differ from the recorded baseline
/// `accuracy --update` overwrite the recorded baseline with the current numbers
fn main() -> std::process::ExitCode {
    match run(std::env::args().nth(1).as_deref()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run(mode: Option<&str>) -> Result<std::process::ExitCode, String> {
    let report = Report::run(&fixtures_dir())?;

    match mode {
        None => print_report(&report),
        Some("--check") => return check(&report),
        Some("--update") => update(&report)?,
        Some(other) => return Err(format!("unknown option {other:?}")),
    }

    Ok(std::process::ExitCode::SUCCESS)
}

fn print_report(report: &Report) {
    println!("{}", report.to_table());

    let details = report.details();
    if !details.is_empty() {
        println!("{details}");
    }
}

fn check(report: &Report) -> Result<std::process::ExitCode, String> {
    let recorded = Baseline::load(&baseline_path())?;
    let current = Baseline::from_report(report);
    let differences = recorded.diff(&current);

    print_report(report);

    if differences.is_empty() {
        println!("accuracy matches the recorded baseline");
        return Ok(std::process::ExitCode::SUCCESS);
    }

    println!("accuracy differs from the recorded baseline:\n");
    differences
        .iter()
        .for_each(|difference| println!("{difference}"));
    println!("\nrun `cargo run -p lintric-accuracy -- --update` to record these numbers");

    Ok(std::process::ExitCode::FAILURE)
}

fn update(report: &Report) -> Result<(), String> {
    Baseline::from_report(report).save(&baseline_path())?;
    print_report(report);
    println!("baseline updated: {}", baseline_path().display());
    Ok(())
}
