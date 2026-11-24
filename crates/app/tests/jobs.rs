//! The pane job runner (ticket 08): a load or relayout task runs on a worker
//! thread and hands its result back over a channel; the UI thread polls once
//! per frame, keeps drawing the old pane until a new result arrives, and
//! discards results older than the newest request.
//!
//! Tests drive the facade without egui or gix: a task is any closure, so
//! tests use the synthetic source with a delay to control which job finishes
//! first. Owned by the mentor; Emil makes these pass by adding
//! `crates/app/src/jobs.rs`.
//!
//! Expected facade:
//! ```rust,ignore
//! // crates/app/src/jobs.rs
//! pub struct PaneJobs { /* generation, channel, outstanding jobs, notify hook */ }
//! impl PaneJobs {
//!     pub fn new() -> Self;
//!     /// Called on the worker thread after a result was sent. The app
//!     //! fills it with `ctx.request_repaint()`; egui only redraws on
//!     //! input otherwise.
//!     pub fn set_notify(&mut self, notify: impl Fn() + Send + 'static);
//!     /// Spawns a worker for this task; each request bumps a generation.
//!     pub fn request(&mut self, task: impl FnOnce() -> Result<Pane, String> + Send + 'static);
//!     /// Drains finished jobs. Returns the newest generation's result;
//!     /// results of older generations are discarded, never surfaced.
//!     /// None when nothing usable arrived.
//!     pub fn poll(&mut self) -> Option<Result<Arc<Pane>, String>>;
//!     /// True while a requested job has not finished yet.
//!     pub fn busy(&self) -> bool;
//! }
//! ```

use groves_app::jobs::PaneJobs;
use groves_core::Pane;
use groves_testkit::{TEST_NOW, SyntheticSource, pane_options};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// A pane job that takes `ms` to run: the real `Pane::load` against the
/// given synthetic source, after a sleep that lets tests control which of
/// two requested jobs finishes first.
fn load_after(ms: u64, source: &SyntheticSource) -> Result<Pane, String> {
    std::thread::sleep(Duration::from_millis(ms));
    Ok(Pane::load(source, &pane_options(TEST_NOW)))
}

/// Poll until the runner hands over a result. Jobs here finish in ~1–150 ms;
/// a full second of polling is generous for CI machines.
fn poll_until_result(jobs: &mut PaneJobs) -> Option<Result<Arc<Pane>, String>> {
    for _ in 0..500 {
        if let Some(result) = jobs.poll() {
            return Some(result);
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    None
}

#[test]
fn a_requested_job_returns_its_result_over_the_channel() {
    let mut jobs = PaneJobs::new();
    jobs.request(|| load_after(1, &SyntheticSource::three()));

    let outcome = poll_until_result(&mut jobs)
        .expect("the job's result must arrive over the channel");

    let pane = outcome.expect("a successful task yields a pane");
    let titles: Vec<&str> = pane.rows().iter().map(|r| r.title.as_str()).collect();
    assert_eq!(
        titles,
        ["Fix lane reuse", "Add filter box", "Initial commit"],
        "the pane the UI receives is the one the task built"
    );
}

#[test]
fn a_stale_result_is_discarded_instead_of_overwriting_the_newest() {
    let mut jobs = PaneJobs::new();
    // Two relayouts back to back, as if the user flipped a toggle twice.
    // The older job is slower and finishes second: its result must never
    // clobber the newer pane.
    jobs.request(|| load_after(120, &SyntheticSource::three()));
    jobs.request(|| load_after(5, &SyntheticSource::linear_three()));

    let outcome = poll_until_result(&mut jobs)
        .expect("the newest job's result must arrive");
    let pane = outcome.expect("the newest job succeeds");
    assert_eq!(
        pane.rows().len(),
        3,
        "the surfaced pane is the *newest* job's (linear_three)"
    );
    assert_eq!(
        pane.rows()[0].title, "Tip",
        "the surfaced pane is the *newest* job's (linear_three)"
    );

    // Give the older job time to finish. Its result exists, but it belongs
    // to a superseded generation: it must not surface.
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        jobs.poll().is_none(),
        "a result older than the newest request must be discarded"
    );
    assert!(
        !jobs.busy(),
        "both jobs have ended; the loading state must be gone"
    );
}

#[test]
fn busy_is_true_while_a_job_runs_and_false_once_its_result_is_polled() {
    let mut jobs = PaneJobs::new();
    assert!(!jobs.busy(), "nothing requested, nothing loading");

    jobs.request(|| load_after(1, &SyntheticSource::three()));
    assert!(
        jobs.busy(),
        "the UI must show its loading state while the job runs"
    );

    poll_until_result(&mut jobs).expect("the job finishes");
    assert!(
        !jobs.busy(),
        "once the result is consumed the loading state clears"
    );
}

#[test]
fn a_finished_job_pokes_the_notify_hook_once() {
    let repaints = Arc::new(AtomicUsize::new(0));
    let mut jobs = PaneJobs::new();
    let count = Arc::clone(&repaints);
    jobs.set_notify(move || {
        count.fetch_add(1, Ordering::SeqCst);
    });

    jobs.request(|| load_after(1, &SyntheticSource::three()));
    poll_until_result(&mut jobs).expect("the job finishes");

    assert_eq!(
        repaints.load(Ordering::SeqCst),
        1,
        "each finished job must poke the UI once, or egui never redraws"
    );
}

#[test]
fn a_failed_task_surfaces_as_an_error_result() {
    let mut jobs = PaneJobs::new();
    jobs.request(|| Err("can't open repo: no such directory".to_string()));

    let outcome = poll_until_result(&mut jobs)
        .expect("a failed task still sends its result over the channel");

    assert_eq!(
        outcome.expect_err("the task failed, so the result is the error"),
        "can't open repo: no such directory"
    );
}
