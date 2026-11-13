//! Ordered presentation of already committed snapshots. Animation never decides acceptance.
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use cube_core::Snapshot;
use cube_core::journal::Turns;
use js::{Promise, promise};
use react::{Dispatch, SetStateAction};

#[derive(Clone)]
pub struct Presentation {
    pub snapshot: Snapshot,
    pub turns: Option<Turns>,
}

struct Waiter {
    revision: u32,
    resolve: &'static dyn Fn(()),
}

pub struct Presentations {
    queue: RefCell<VecDeque<Presentation>>,
    target: RefCell<Snapshot>,
    completed: Cell<u32>,
    waiters: RefCell<Vec<Waiter>>,
    publish: &'static Dispatch<SetStateAction<Presentation>>,
}

impl Presentations {
    pub fn new(
        initial: Snapshot,
        publish: &'static Dispatch<SetStateAction<Presentation>>,
    ) -> Self {
        Self {
            queue: RefCell::new(VecDeque::new()),
            completed: Cell::new(initial.revision),
            target: RefCell::new(initial),
            waiters: RefCell::new(Vec::new()),
            publish,
        }
    }

    pub fn enqueue(&self, snapshot: Snapshot, turns: Option<Turns>) {
        let target = self.target.borrow();
        if snapshot.revision <= target.revision {
            return;
        }
        let turns = turns.filter(|turns| {
            snapshot.revision == target.revision + 1 && turns.from == target.facelets
        });
        drop(target);
        *self.target.borrow_mut() = snapshot.clone();
        let first = self.queue.borrow().is_empty();
        let step = Presentation { snapshot, turns };
        self.queue.borrow_mut().push_back(step.clone());
        if first {
            self.publish.set(step);
        }
    }

    pub fn finish(&self, revision: u32) {
        if self
            .queue
            .borrow()
            .front()
            .is_some_and(|step| step.snapshot.revision == revision)
        {
            self.queue.borrow_mut().pop_front();
            self.completed.set(revision);
            self.resolve_waiters();
            let next = self.queue.borrow().front().cloned();
            if let Some(next) = next {
                self.publish.set(next);
            }
        }
    }

    pub fn interrupt(&self) {
        self.queue.borrow_mut().clear();
        self.completed.set(self.target.borrow().revision);
        self.resolve_waiters();
    }

    pub async fn wait(&self, revision: u32) {
        let waiting: Promise<()> = if revision <= self.completed.get() {
            promise::resolve(())
        } else {
            let waiting = promise::with_resolvers();
            self.waiters.borrow_mut().push(Waiter {
                revision,
                resolve: waiting.resolve,
            });
            waiting.promise
        };
        waiting.await;
    }

    fn resolve_waiters(&self) {
        let waiters = std::mem::take(&mut *self.waiters.borrow_mut());
        let mut ready = Vec::new();
        for waiter in waiters {
            if waiter.revision <= self.completed.get() {
                ready.push(waiter.resolve);
            } else {
                self.waiters.borrow_mut().push(waiter);
            }
        }
        for resolve in ready {
            resolve(());
        }
    }
}
