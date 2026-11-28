use groves_core::Pane;
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, mpsc};
use std::thread;

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
    pub fn set_notify(&mut self, notify: impl Fn() + Send + Sync + 'static) {
        self.notify = Some(Arc::new(notify));
    }
    pub fn request(&mut self, task: impl FnOnce() -> Result<Pane, String> + Send + 'static) {
        let tx = self.result_tx.clone();
        self.generation += 1;
        let this_generation = self.generation;
        self.outstanding.insert(this_generation);
        let notify = self.notify.clone();
        thread::spawn(move || {
            let content = task();
            let result = JobOutcome {
                result: content.map(Arc::new),
                generation: this_generation,
            };
            let _ = tx.send(result);
            if let Some(notify) = notify {
                notify()
            };
        });
    }
    pub fn poll(&mut self) -> Option<Result<Arc<Pane>, String>> {
        let mut current: Option<Result<Arc<Pane>, String>> = None;
        while let Ok(outcome) = self.result_rx.try_recv() {
            self.outstanding.remove(&outcome.generation);
            if self.generation == outcome.generation {
                current = Some(outcome.result);
            }
        }
        current
    }
    pub fn busy(&self) -> bool {
        !self.outstanding.is_empty()
    }
}

impl Default for PaneJobs {
    fn default() -> Self {
        Self::new()
    }
}
