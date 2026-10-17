//! Printing: diagnostics through miette, progress through console.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use console::style;
use miette::{Diagnostic, Report};

/// Print an error or warning with its help.
pub fn error<D: Diagnostic + Send + Sync + 'static>(diagnostic: D) {
    eprintln!("{:?}", Report::new(diagnostic));
}

/// Print a line of progress about a repo, unless `--quiet` was given.
pub fn progress(quiet: bool, path: &str, message: &str) {
    if !quiet {
        eprintln!("{} {message}", style(path).bold());
    }
}

/// Run `f` on every item with up to `jobs` threads, and return the results
/// in the order of the items.
pub fn parallel<T: Sync, R: Send>(items: &[T], jobs: usize, f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let next = AtomicUsize::new(0);
    let results: Vec<Mutex<Option<R>>> = items.iter().map(|_| Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        for _ in 0..jobs.min(items.len()) {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(item) = items.get(i) else { break };
                    let result = f(item);
                    *results[i].lock().expect("result lock") = Some(result);
                }
            });
        }
    });
    results
        .into_iter()
        .map(|r| {
            r.into_inner()
                .expect("result lock")
                .expect("every item has a result")
        })
        .collect()
}
