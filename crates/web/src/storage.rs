//! One transaction is the acceptance boundary. Never publish a proposal before completion.
use std::cell::RefCell;
use std::panic::catch_unwind;
use std::rc::Rc;

use cube_core::Snapshot;
use cube_core::journal::{Acceptance, Command, Journal};
use js::{promise, settle};
use webapi::AbortSignal;

use crate::idb::{self, Database, TransactionOptions};

const MAX_JSON_BYTES: usize = 4 * 1024 * 1024;
const STORE: &str = "practice";
const KEY: &str = "active";

#[derive(Clone)]
pub enum Storage {
    IndexedDb(&'static Database),
    Temporary(Rc<RefCell<Journal>>),
}

pub async fn open() -> Result<Storage, String> {
    let opening = catch_unwind(|| {
        idb::open_db(
            "agent-cube",
            1,
            idb::OpenOptions {
                upgrade: |db: &'static Database, _, _, _, _| {
                    db.create_object_store(STORE);
                },
            },
        )
    });
    match opening {
        Ok(opening) => match settle(opening).await {
            Ok(db) => {
                db.set_on_version_change(move |_| db.close());
                Ok(Storage::IndexedDb(db))
            }
            Err(error) => {
                if error.name() == "VersionError" {
                    Err("A newer app saved this database. Update or reload AgentCube; saved data was preserved.".into())
                } else {
                    Ok(Storage::Temporary(Rc::new(
                        RefCell::new(Journal::default()),
                    )))
                }
            }
        },
        Err(_) => Ok(Storage::Temporary(Rc::new(
            RefCell::new(Journal::default()),
        ))),
    }
}

fn decode(text: Option<String>) -> Result<Journal, String> {
    match text {
        Some(text) if text.len() <= MAX_JSON_BYTES => serde_json::from_str(&text).map_err(|_| {
            "Saved session is unreadable. It has been preserved; no moves were accepted.".into()
        }),
        Some(_) => Err("Saved session exceeds the supported size. It has been preserved.".into()),
        None => Ok(Journal::default()),
    }
}

fn cancelled(signal: Option<&AbortSignal>) -> bool {
    signal.is_some_and(|signal| signal.aborted())
}

impl Storage {
    pub fn durable(&self) -> bool {
        matches!(self, Self::IndexedDb(_))
    }

    async fn transact(
        &self,
        command: Option<Command>,
        signal: Option<&'static AbortSignal>,
    ) -> Result<Acceptance, String> {
        if cancelled(signal) {
            return Err("Request cancelled before acceptance.".into());
        }
        let seed = (js::math::random() * f64::from(u32::MAX)) as u32;
        match self {
            Self::Temporary(saved) => {
                let mut proposal = saved.borrow().clone();
                let accepted = propose(&mut proposal, command, seed)?;
                *saved.borrow_mut() = proposal;
                Ok(accepted)
            }
            Self::IndexedDb(db) => {
                let mode = if command.is_some() {
                    "readwrite"
                } else {
                    "readonly"
                };
                let transaction = catch_unwind(|| {
                    db.transaction(
                        STORE,
                        mode,
                        TransactionOptions {
                            durability: "strict",
                        },
                    )
                })
                .map_err(|_| {
                    "Saved session connection is closed. Reload before continuing.".to_string()
                })?;
                // Install the rejection handler before requests: aborts must never become unhandled rejections.
                let completion = transaction.done().then_with(
                    |_| Ok(()),
                    |_| Err("Saving failed. No command was committed; retry with the same commandId.".to_string()),
                );
                let operation = promise(async move {
                    let store = transaction.object_store(STORE);
                    let text = settle(store.get::<String>(KEY))
                        .await
                        .map_err(|_| "Could not read the saved session.".to_string())?;
                    let mut proposal = decode(text)?;
                    let accepted = propose(&mut proposal, command, seed)?;
                    if accepted.accepted {
                        if cancelled(signal) {
                            return Err("Request cancelled before acceptance.".into());
                        }
                        let text = serde_json::to_string(&proposal)
                            .map_err(|_| "Could not encode the session.".to_string())?;
                        if text.len() > MAX_JSON_BYTES {
                            return Err(
                                "Saved history is full. The last committed state was retained."
                                    .into(),
                            );
                        }
                        settle(store.put(text, KEY)).await.map_err(|_| {
                            "Could not save the command. Retry with the same commandId.".to_string()
                        })?;
                    }
                    Ok(accepted)
                });
                let result = settle(operation)
                    .await
                    .map_err(|_| "Storage failed. Retry with the same commandId.".to_string());
                // A request succeeding is not a commit. Wait even when a request failed.
                completion.await?;
                result?
            }
        }
    }

    pub async fn read(&self) -> Result<Snapshot, String> {
        self.transact(None, None).await.map(|result| result.current)
    }
    pub async fn submit(
        &self,
        command: Command,
        signal: Option<&'static AbortSignal>,
    ) -> Result<Acceptance, String> {
        self.transact(Some(command), signal).await
    }
}

fn propose(
    journal: &mut Journal,
    command: Option<Command>,
    seed: u32,
) -> Result<Acceptance, String> {
    match command {
        Some(command) => journal.accept(command, seed),
        None => {
            let snapshot = journal.snapshot()?;
            Ok(Acceptance {
                receipt: snapshot.clone(),
                current: snapshot,
                accepted: false,
            })
        }
    }
}
