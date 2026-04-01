use anyhow::Context;
use indicatif::ProgressBar;
use nonempty::NonEmpty;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::{Sender, channel},
};

use crate::walk;

#[derive(Deserialize)]
pub(crate) struct ReportSpec {
    categories: Vec<CategorySpec>,
}

#[derive(Deserialize)]
struct CategorySpec {
    name: String,
    command: NonEmpty<String>,

    #[serde(default)]
    stats: Vec<StatSpec>,
}

#[derive(Deserialize)]
struct StatSpec {
    name: String,
    command: NonEmpty<String>,
}

impl ReportSpec {
    pub(crate) fn load(spec_path: &Path) -> anyhow::Result<ReportSpec> {
        let read_to_string = std::fs::read_to_string(spec_path)?;
        let mut spec: ReportSpec = toml::from_str(&read_to_string)
            .with_context(|| format!("Bad config file {}", spec_path.display()))?;

        let spec_dir = spec_path.parent().unwrap();
        let resolve_relative_path = |command: &mut NonEmpty<String>| {
            // This implements shell semantics for looking up commands. If there's no slash, it's a
            // PATH lookup. If there's a slash, it's either a relative or absolute path.
            let has_slash = command.head.contains("/");
            if has_slash {
                let program = PathBuf::from(&command.head);
                if program.is_relative() {
                    let resolved = spec_dir.join(program).canonicalize().unwrap();
                    command.head = resolved.to_str().unwrap().to_string();
                }
            }
        };

        // Normalize all relatives paths to be absolute paths relative
        // to the location of the config file.
        for category_spec in spec.categories.iter_mut() {
            resolve_relative_path(&mut category_spec.command);
            for stat in category_spec.stats.iter_mut() {
                resolve_relative_path(&mut stat.command);
            }
        }
        Ok(spec)
    }
}

fn build_command(args: &NonEmpty<&str>) -> Command {
    let mut command = Command::new(args.head);
    command.args(&args.tail);
    command
}

fn run_capture_stdout(args: &NonEmpty<&str>, cwd: &PathBuf) -> String {
    let child = build_command(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("{} failed to start", args.head))
        .unwrap()
        .wait_with_output()
        .unwrap();

    assert!(child.status.success(), "{} command failed", args.head);

    str::from_utf8(&child.stdout).unwrap().into()
}

pub(crate) struct ReportRunner {
    report_spec: ReportSpec,
}

pub struct Report {
    stat_names: Vec<String>,
    items: Vec<ReportItem>,
}

impl Report {
    pub(crate) fn write_csv(&self, writer: impl std::io::Write) -> anyhow::Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        // First, the header.
        wtr.write_field("path")?;
        wtr.write_field("category")?;
        wtr.write_record(&self.stat_names)?;

        // Now the rows.
        for item in &self.items {
            wtr.write_field(item.path.to_string_lossy().to_string())?;
            wtr.write_field(&item.category)?;
            for stat_name in &self.stat_names {
                wtr.write_field(item.stats.get(stat_name).unwrap_or(&"".to_string()))?;
            }
            wtr.write_record(None::<&[u8]>)?;
        }

        wtr.flush()?;

        Ok(())
    }
}

struct ReportItem {
    path: PathBuf,
    category: String,
    stats: HashMap<String, String>,
}

enum WalkMessage<'a> {
    MiscFile(PathBuf),
    CategorizedDir(&'a CategorySpec, PathBuf),
}

impl ReportRunner {
    fn categorize(&self, dir: &Path) -> Option<&CategorySpec> {
        for category_spec in &self.report_spec.categories {
            let cp = build_command(&category_spec.command.as_ref().map(String::as_str))
                .current_dir(dir)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .with_context(|| format!("{} failed to start", category_spec.command.head))
                .unwrap()
                .wait_with_output()
                .unwrap();

            if cp.status.success() {
                return Some(category_spec);
            }
        }

        None
    }

    fn walk<'a>(&'a self, tx: Sender<WalkMessage<'a>>, roots: &[PathBuf]) -> anyhow::Result<()> {
        if roots.is_empty() {
            anyhow::bail!("Must specify at least one root to process");
        }

        let bad_roots: Vec<_> = roots
            .iter()
            .filter(|root| !(root.exists() && root.is_dir()))
            .collect();

        if !bad_roots.is_empty() {
            anyhow::bail!("Roots must exist and be directories. These are not: {bad_roots:?}");
        }

        walk::walk(
            roots,
            tx,
            |file, tx| {
                tx.send(WalkMessage::MiscFile(file.to_path_buf())).unwrap();
            },
            |dir, tx| {
                if let Some(category) = self.categorize(dir) {
                    tx.send(WalkMessage::CategorizedDir(category, dir.to_path_buf()))
                        .unwrap();
                    walk::Decision::Stop
                } else {
                    walk::Decision::Continue
                }
            },
        );

        Ok(())
    }

    pub(crate) fn run(&self, roots: &[PathBuf]) -> anyhow::Result<Report> {
        let pb = ProgressBar::no_length();
        pb.set_position(0);

        let mut msgs: Vec<WalkMessage> = Vec::new();
        rayon::scope(|scope| -> anyhow::Result<()> {
            let (tx_walk, rx_walk) = channel::<WalkMessage>();

            scope.spawn(|_scope| {
                for msg in rx_walk {
                    msgs.push(msg);
                    pb.set_length(pb.length().unwrap_or(0) + 1);
                }
            });

            self.walk(tx_walk, roots)?;

            Ok(())
        })?;

        let (item_tx, item_rx) = channel::<ReportItem>();

        msgs.into_par_iter().for_each(|msg| {
            match msg {
                WalkMessage::MiscFile(file) => {
                    item_tx
                        .send(ReportItem {
                            path: file.to_path_buf(),
                            category: "misc".to_string(),
                            stats: HashMap::new(),
                        })
                        .unwrap();
                }
                WalkMessage::CategorizedDir(category_spec, dir) => {
                    let stats: HashMap<String, String> = category_spec
                        .stats
                        .par_iter()
                        .map(|stat_spec| {
                            let args = &stat_spec.command.as_ref().map(String::as_str);
                            let stat = run_capture_stdout(args, &dir);
                            (stat_spec.name.clone(), stat.trim().into())
                        })
                        .collect();
                    item_tx
                        .send(ReportItem {
                            path: dir.to_path_buf(),
                            category: category_spec.name.clone(),
                            stats,
                        })
                        .unwrap();
                }
            }
            pb.set_position(pb.position() + 1);
        });
        drop(item_tx);

        pb.finish_and_clear();

        let stat_names: Vec<_> = self
            .report_spec
            .categories
            .iter()
            .flat_map(|category| category.stats.iter().map(|stat| stat.name.clone()))
            .collect();
        let report = Report {
            stat_names,
            items: item_rx.iter().collect(),
        };
        Ok(report)
    }

    pub(crate) fn new(report_spec: ReportSpec) -> Self {
        Self { report_spec }
    }
}
