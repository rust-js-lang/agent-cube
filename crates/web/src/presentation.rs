//! Ordered presentation of committed state; replay is a separate read-only view.
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use cube_core::Snapshot;
use cube_core::journal::{Playback, Turns};
use js::{Promise, promise};
use react::{Dispatch, SetStateAction};

#[derive(Clone)]
pub struct Presentation {
    /// Presentation identity is independent of game revision, so replay can repeat.
    pub id: u32,
    pub replay: bool,
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
    next_id: Cell<u32>,
    waiters: RefCell<Vec<Waiter>>,
    replay_completion: RefCell<Option<&'static dyn Fn(())>>,
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
            next_id: Cell::new(1),
            waiters: RefCell::new(Vec::new()),
            replay_completion: RefCell::new(None),
            publish,
        }
    }

    fn step(&self, snapshot: Snapshot, turns: Option<Turns>, replay: bool) -> Presentation {
        let id = self.next_id.get();
        self.next_id.set(id.wrapping_add(1));
        Presentation {
            id,
            replay,
            snapshot,
            turns,
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
        // New committed play takes precedence over a read-only replay.
        if self.queue.borrow().front().is_some_and(|step| step.replay) {
            self.queue.borrow_mut().pop_front();
            self.resolve_replay();
        }
        *self.target.borrow_mut() = snapshot.clone();
        let first = self.queue.borrow().is_empty();
        let step = self.step(snapshot, turns, false);
        self.queue.borrow_mut().push_back(step.clone());
        if first {
            self.publish.set(step);
        }
    }

    pub fn finish(&self, id: u32) {
        if self
            .queue
            .borrow()
            .front()
            .is_some_and(|step| step.id == id)
        {
            let step = self
                .queue
                .borrow_mut()
                .pop_front()
                .expect("checked presentation");
            if step.replay {
                self.resolve_replay();
            } else {
                self.completed.set(step.snapshot.revision);
                self.resolve_waiters();
            }
            let next = self.queue.borrow().front().cloned();
            if let Some(next) = next {
                self.publish.set(next);
            }
        }
    }

    pub async fn replay(&self, playback: Playback) -> Result<(), String> {
        let matches = {
            let current = self.target.borrow();
            playback.snapshot.revision == current.revision
                && playback.snapshot.facelets == current.facelets
        };
        if !matches || !self.queue.borrow().is_empty() {
            Err("The cube changed. Try replay again after the current moves finish.".into())
        } else if playback.turns.moves.is_empty() {
            Err("Play some moves before replaying this challenge.".into())
        } else {
            let waiting = promise::with_resolvers();
            *self.replay_completion.borrow_mut() = Some(waiting.resolve);
            let step = self.step(playback.snapshot, Some(playback.turns), true);
            self.queue.borrow_mut().push_back(step.clone());
            self.publish.set(step);
            waiting.promise.await;
            Ok(())
        }
    }

    pub fn interrupt(&self) {
        self.queue.borrow_mut().clear();
        self.completed.set(self.target.borrow().revision);
        self.resolve_replay();
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

    fn resolve_replay(&self) {
        let resolve = self.replay_completion.borrow_mut().take();
        if let Some(resolve) = resolve {
            resolve(());
        }
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
