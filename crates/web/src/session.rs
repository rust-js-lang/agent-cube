//! Browser adapter: human controls and tools share one local referee.
use std::cell::RefCell;

use cube_core::{Session, Snapshot};
use react::{Dispatch, SetStateAction, dom::flush_sync};

pub struct LocalSession {
    state: RefCell<Session>,
    publish: &'static Dispatch<SetStateAction<Snapshot>>,
}

impl LocalSession {
    pub fn new(publish: &'static Dispatch<SetStateAction<Snapshot>>) -> Self {
        Self {
            state: RefCell::new(Session::default()),
            publish,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.state.borrow().snapshot()
    }

    pub fn apply(&self, moves: &str, revision: u32) -> Result<Snapshot, String> {
        let result = self.state.borrow_mut().apply_moves(moves, revision);
        self.present(result)
    }

    pub fn shuffle(&self, length: usize) -> Result<Snapshot, String> {
        let seed = (js::math::random() * f64::from(u32::MAX)) as u32;
        let result = self.state.borrow_mut().shuffle(seed, length);
        self.present(result)
    }

    fn present(&self, result: Result<Snapshot, String>) -> Result<Snapshot, String> {
        if let Ok(snapshot) = &result {
            let snapshot = snapshot.clone();
            let publish = self.publish;
            flush_sync(move || publish.set(snapshot));
        }
        result
    }
}
