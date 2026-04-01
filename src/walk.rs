/// Utility to walk the filesystem in parallel.
/// This is a thin wrapper on top of `rayon`.
/// Supports stopping recursion into subdirectories.
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

pub(crate) enum Decision {
    Continue,
    Stop,
}

pub(crate) fn walk<C: Clone + Send + Sync>(
    roots: &[PathBuf],
    context: C,
    visit_file: impl Fn(&Path, C) + Sync + Send,
    visit_dir: impl Fn(&Path, C) -> Decision + Sync + Send,
) {
    fn handle_path<C: Clone + Send + Sync>(
        context: &C,
        visit_file: impl Fn(&Path, C) + Sync + Send + Copy,
        visit_dir: impl Fn(&Path, C) -> Decision + Sync + Send + Copy,
        root: &Path,
    ) {
        let is_dir = root.is_dir();
        if is_dir {
            match visit_dir(root, context.clone()) {
                Decision::Continue => run_inner(root, context.clone(), visit_file, visit_dir),
                Decision::Stop => {}
            }
        } else {
            visit_file(root, context.clone());
        }
    }

    fn run_inner<C: Clone + Send + Sync>(
        root: &Path,
        context: C,
        visit_file: impl Fn(&Path, C) + Sync + Send + Copy,
        visit_dir: impl Fn(&Path, C) -> Decision + Sync + Send + Copy,
    ) {
        let entries: Vec<_> = read_dir(root).unwrap().map(|e| e.unwrap()).collect();

        entries.into_par_iter().for_each(|entry| {
            let path = entry.path();
            let is_dir = entry.file_type().unwrap().is_dir();

            if is_dir {
                match visit_dir(&path, context.clone()) {
                    Decision::Continue => run_inner(&path, context.clone(), visit_file, visit_dir),
                    Decision::Stop => {}
                }
            } else {
                visit_file(&path, context.clone());
            }
        });
    }

    for root in roots {
        handle_path(&context, &visit_file, &visit_dir, root);
    }
}
