use std::path::PathBuf;

use clap::Parser;

use crate::report::{ReportRunner, ReportSpec};

mod report;
mod walk;

/// Tool to search for code repositories and delete clean ones.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to report spec.
    report_spec_path: PathBuf,

    /// Root path to process. Can be specified multiple times.
    #[arg(name = "root", short, long)]
    roots: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let report_spec = ReportSpec::load(&args.report_spec_path)?;

    let runner = ReportRunner::new(report_spec);
    let report = runner.run(&args.roots)?;

    report.write_csv(std::io::stdout())?;

    Ok(())
}
