//! Browser session boundary shared by human controls and WebMCP.
use std::cell::RefCell;
use std::panic::catch_unwind;
use std::rc::Rc;

use cube_core::journal::{Action, Command, Turns};
use cube_core::{Cube, Session, Snapshot};
use react::{Dispatch, SetStateAction, dom::flush_sync};
use webapi::{AbortSignal, BroadcastChannel, window};

use crate::presentation::{Presentation, Presentations};
use crate::storage::{self, Storage};

pub struct LocalSession {
    state: RefCell<Snapshot>,
    pub presentations: Rc<Presentations>,
    storage: RefCell<Option<Storage>>,
    channel: RefCell<Option<&'static BroadcastChannel>>,
    publish: &'static Dispatch<SetStateAction<Snapshot>>,
    status: &'static Dispatch<SetStateAction<&'static str>>,
}

impl LocalSession {
    pub fn new(
        publish: &'static Dispatch<SetStateAction<Snapshot>>,
        status: &'static Dispatch<SetStateAction<&'static str>>,
        presentation: &'static Dispatch<SetStateAction<Presentation>>,
    ) -> Self {
        Self {
            state: RefCell::new(Session::default().snapshot()),
            presentations: Rc::new(Presentations::new(
                Session::default().snapshot(),
                presentation,
            )),
            storage: RefCell::new(None),
            channel: RefCell::new(None),
            publish,
            status,
        }
    }

    pub fn ready(&self) -> bool {
        self.storage.borrow().is_some()
    }
    pub fn snapshot(&self) -> Snapshot {
        self.state.borrow().clone()
    }
    pub fn set_channel(&self, channel: Option<&'static BroadcastChannel>) {
        *self.channel.borrow_mut() = channel;
    }

    pub async fn initialize(&self) -> Result<(), String> {
        let linked = linked_position()?;
        let storage = storage::open().await?;
        let mut snapshot = storage.read().await?;
        let action = entry_action(linked, &snapshot);
        let changed = action.is_some();
        if let Some(action) = action {
            snapshot = storage
                .submit(
                    Command {
                        command_id: window.crypto().random_uuid(),
                        expected_revision: snapshot.revision,
                        action,
                    },
                    None,
                )
                .await?
                .current;
        }
        self.status.set(if storage.durable() {
            "Saved on this browser"
        } else {
            "Temporary · history not saved"
        });
        *self.storage.borrow_mut() = Some(storage);
        self.present(snapshot, None);
        if changed {
            self.notify();
        }
        Ok(())
    }

    fn backend(&self) -> Result<Storage, String> {
        self.storage
            .borrow()
            .clone()
            .ok_or_else(|| "Saved session is not ready. Check the browser storage status.".into())
    }

    pub async fn read(&self) -> Result<Snapshot, String> {
        let snapshot = self.backend()?.read().await?;
        self.present(snapshot.clone(), None);
        self.presentations.wait(snapshot.revision).await;
        Ok(snapshot)
    }

    pub async fn replay(&self) -> Result<(), String> {
        let playback = self.backend()?.playback().await?;
        let revision = playback.snapshot.revision;
        self.present(playback.snapshot.clone(), None);
        self.presentations.wait(revision).await;
        self.presentations.replay(playback).await
    }

    pub async fn submit(
        &self,
        command: Command,
        signal: Option<&'static AbortSignal>,
    ) -> Result<Snapshot, String> {
        let storage = self.backend()?;
        match storage.submit(command, signal).await {
            Ok(accepted) => {
                let revision = accepted.current.revision;
                self.present(accepted.current, accepted.turns);
                if accepted.accepted {
                    self.notify();
                }
                self.presentations.wait(revision).await;
                Ok(accepted.receipt)
            }
            Err(error) => {
                // Refresh stale plans, but never replace saved play with temporary play after a failure.
                if let Ok(current) = storage.read().await {
                    self.present(current, None);
                }
                Err(error)
            }
        }
    }

    pub async fn load_link(&self) -> Result<(), String> {
        let linked = linked_position()?;
        let current = self.backend()?.read().await?;
        if let Some(action) = entry_action(linked, &current) {
            self.submit(
                Command {
                    command_id: window.crypto().random_uuid(),
                    expected_revision: current.revision,
                    action,
                },
                None,
            )
            .await?;
        }
        Ok(())
    }

    fn notify(&self) {
        if let Some(channel) = *self.channel.borrow() {
            let _ = catch_unwind(|| channel.post_message("committed"));
        }
    }

    fn present(&self, snapshot: Snapshot, turns: Option<Turns>) {
        if snapshot.revision >= self.state.borrow().revision {
            let target = format!("#cube={}", snapshot.facelets);
            if window.location().hash() != target {
                let _ = window
                    .history()
                    .replace_state("AgentCube", "", Some(&target));
            }
            *self.state.borrow_mut() = snapshot.clone();
            let publish = self.publish;
            let presentations = Rc::clone(&self.presentations);
            flush_sync(move || {
                publish.set(snapshot.clone());
                presentations.enqueue(snapshot, turns);
            });
        }
    }
}

fn linked_position() -> Result<Option<String>, String> {
    let hash = window.location().hash();
    if hash.is_empty() {
        Ok(None)
    } else {
        let position = hash.strip_prefix("#cube=").ok_or("Invalid cube link. Expected #cube= followed by 54 URFDLB stickers. Saved data was preserved.")?;
        Cube::from_facelets(position)?;
        Ok(Some(position.into()))
    }
}

// A supplied position resumes/loads that challenge; a bare entry always starts fresh.
fn entry_action(linked: Option<String>, snapshot: &Snapshot) -> Option<Action> {
    match linked {
        None => Some(Action::Shuffle(20)),
        Some(position) if position != snapshot.facelets => Some(Action::Load(position)),
        Some(_) => None,
    }
}
