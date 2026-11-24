use groves_core::Pane;
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, mpsc};

struct JobOutcome {
    generation: u64,
    result: Result<Arc<Pane>, String>,
}
pub struct PaneJobs {
    generation: u64,
    outstanding: HashSet<u64>,
    notify: Option<Arc<dyn Fn() + Send + Sync>>,
    result_tx: Sender<JobOutcome>,
    result_rx: Receiver<JobOutcome>,
}
impl PaneJobs {
    pub fn new() -> Self {
        let (result_tx, result_rx) = mpsc::channel();
        Self {
            generation: 0,
            outstanding: HashSet::new(),
            notify: None,
            result_tx,
            result_rx,
        }
    }
    pub fn set_notify(&mut self, notify: impl Fn() + Send + 'static) {} // runs on the worker thread
    pub fn request(&mut self, task: impl FnOnce() -> Result<Pane, String> + Send + 'static) {}
    pub fn poll(&mut self) -> Option<Result<Arc<Pane>, String>> {
        None
    } // drains, drops stale generations
    pub fn busy(&self) -> bool {
        !self.outstanding.is_empty()
    }
}

impl Default for PaneJobs {
    fn default() -> Self {
        Self::new()
    }
}
