//! Replayable local practice log. Storage commits a proposed log before publishing it.
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{Session, Snapshot};

pub const MAX_COMMANDS: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub command_id: String,
    pub expected_revision: u32,
    pub action: Action,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Moves(String),
    Load(String),
    Shuffle(usize),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    command: Command,
    seed: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Journal {
    version: u32,
    entries: Vec<Entry>,
}

pub struct Acceptance {
    /// The original receipt, including when retried after newer commands.
    pub receipt: Snapshot,
    /// The latest authoritative state, which the UI must display.
    pub current: Snapshot,
    pub accepted: bool,
}

impl Default for Journal {
    fn default() -> Self {
        Self {
            version: 1,
            entries: Vec::new(),
        }
    }
}

impl Command {
    fn validate(&self) -> Result<(), String> {
        if self.command_id.is_empty() || self.command_id.len() > 128 || !self.command_id.is_ascii()
        {
            Err("commandId must contain 1–128 ASCII characters.".into())
        } else {
            Ok(())
        }
    }
}

fn apply(session: &mut Session, entry: &Entry) -> Result<Snapshot, String> {
    entry.command.validate()?;
    if session.snapshot().revision != entry.command.expected_revision {
        Err("Stale revision. Read the cube again before submitting a new command.".into())
    } else {
        match (&entry.command.action, entry.seed) {
            (Action::Load(facelets), None) => {
                session.load(facelets, entry.command.expected_revision)
            }
            (Action::Moves(moves), None) => {
                session.apply_moves(moves, entry.command.expected_revision)
            }
            (Action::Shuffle(length), Some(seed)) => session.shuffle(seed, *length),
            _ => Err("Invalid saved command seed.".into()),
        }
    }
}

impl Journal {
    fn replay(&self) -> Result<(Session, Vec<Snapshot>), String> {
        if self.version != 1 || self.entries.len() > MAX_COMMANDS {
            Err("Unsupported saved session format or size. Saved data has not been changed.".into())
        } else {
            let mut session = Session::default();
            let mut ids = HashSet::new();
            let mut receipts = Vec::with_capacity(self.entries.len());
            for entry in &self.entries {
                if !ids.insert(entry.command.command_id.clone()) {
                    return Err("Duplicate command in saved session.".into());
                }
                receipts.push(apply(&mut session, entry)?);
            }
            Ok((session, receipts))
        }
    }

    pub fn snapshot(&self) -> Result<Snapshot, String> {
        self.replay().map(|(session, _)| session.snapshot())
    }

    /// This changes only the proposed journal; the adapter must commit it atomically.
    pub fn accept(&mut self, command: Command, seed: u32) -> Result<Acceptance, String> {
        command.validate()?;
        let (mut session, receipts) = self.replay()?;
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.command.command_id == command.command_id)
        {
            if self.entries[index].command != command {
                Err("commandId was already used with different content.".into())
            } else {
                Ok(Acceptance {
                    receipt: receipts[index].clone(),
                    current: session.snapshot(),
                    accepted: false,
                })
            }
        } else if self.entries.len() == MAX_COMMANDS {
            Err("This practice log has reached its 4096-command limit. Saved commands have been retained.".into())
        } else {
            let seed = match &command.action {
                Action::Moves(_) | Action::Load(_) => None,
                Action::Shuffle(_) => Some(seed),
            };
            let entry = Entry { command, seed };
            let receipt = apply(&mut session, &entry)?;
            self.entries.push(entry);
            Ok(Acceptance {
                current: receipt.clone(),
                receipt,
                accepted: true,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moves(id: &str, revision: u32, moves: &str) -> Command {
        Command {
            command_id: id.into(),
            expected_revision: revision,
            action: Action::Moves(moves.into()),
        }
    }

    #[test]
    fn retries_return_original_receipt_without_rewinding_current_state() {
        let mut log = Journal::default();
        let first = log.accept(moves("one", 0, "R"), 0).unwrap().receipt;
        log.accept(moves("two", 1, "U"), 0).unwrap();
        let retry = log.accept(moves("one", 0, "R"), 123).unwrap();
        assert!(!retry.accepted);
        assert_eq!(retry.receipt.facelets, first.facelets);
        assert_eq!(retry.receipt.revision, 1);
        assert_eq!(retry.current.revision, 2);
        assert_eq!(retry.current.move_count, 2);
        assert_eq!(log.entries.len(), 2);
    }

    #[test]
    fn collisions_stale_invalid_and_cancelled_proposals_do_not_change_saved_log() {
        let mut saved = Journal::default();
        saved.accept(moves("one", 0, "R"), 0).unwrap();
        let before = saved.snapshot().unwrap();
        for command in [
            moves("one", 0, "U"),
            moves("two", 0, "U"),
            moves("two", 1, "U invalid"),
            moves("", 1, "U"),
        ] {
            assert!(saved.accept(command, 0).is_err());
            assert_eq!(saved.snapshot().unwrap().facelets, before.facelets);
        }
        let mut proposal = saved.clone();
        proposal.accept(moves("two", 1, "U"), 0).unwrap();
        // A failed transaction discards the proposal, not the accepted journal.
        assert_eq!(saved.snapshot().unwrap().revision, 1);
    }

    #[test]
    fn shuffle_retry_keeps_seed_and_replay_restores_all_state() {
        let mut log = Journal::default();
        let shuffle = Command {
            command_id: "shuffle".into(),
            expected_revision: 0,
            action: Action::Shuffle(20),
        };
        let first = log.accept(shuffle.clone(), 42).unwrap();
        let retry = log.accept(shuffle, 999).unwrap();
        assert_eq!(first.receipt.facelets, retry.receipt.facelets);
        log.accept(moves("one", 1, "R U2 F'"), 0).unwrap();
        let restored = log.clone().snapshot().unwrap();
        assert_eq!(restored.revision, 2);
        assert_eq!(restored.move_count, 3);
        assert_eq!(restored.facelets, log.snapshot().unwrap().facelets);
    }

    #[test]
    fn corrupt_and_future_logs_are_rejected() {
        let mut log = Journal {
            version: 2,
            ..Journal::default()
        };
        assert!(log.snapshot().is_err());
        log.version = 1;
        log.entries.push(Entry {
            command: moves("one", 1, "R"),
            seed: None,
        });
        assert!(log.snapshot().is_err());
        log.entries[0].command.expected_revision = 0;
        log.entries[0].seed = Some(1);
        assert!(log.snapshot().is_err());
        log.entries[0].seed = None;
        log.entries.push(log.entries[0].clone());
        assert!(log.snapshot().is_err());
    }
}
