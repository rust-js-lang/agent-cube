use std::panic::catch_unwind;
use std::rc::Rc;

mod colors;
mod idb;
mod presentation;
mod preview;
mod session;
mod storage;
mod three;
mod webmcp;

use cube_core::journal::{Action, Command};
use cube_core::{FACES, Session, Snapshot};
use js::{Kind, Unknown, classify, object, promise, settle, spawn};
use react::{CSSProperties, JSX, dom::flush_sync, jsx, use_effect, use_ref, use_state};
use webapi::{Event, EventTargetExt, HashChangeEvent, broadcast_channel, events, window};

use colors::color;
use presentation::Presentation;
use preview::Preview;
use session::LocalSession;
use webmcp::{
    Annotations, ExecutionOptions, IntegerProperty, Properties, Schema, TextProperty, ToolOptions,
    ToolResponse, ViewProperty, use_web_mcp,
};

fn valid_keys(input: Option<&Unknown>, required: &[&str], optional: &[&str]) -> bool {
    match classify(input) {
        Kind::Object(_) => {
            let input = input.expect("object input");
            object::keys(input)
                .iter()
                .all(|key| required.contains(&key.as_str()) || optional.contains(&key.as_str()))
                && required.iter().all(|key| object::has_own(input, key))
        }
        _ => false,
    }
}

fn integer(input: Option<&Unknown>, key: &str) -> Option<u32> {
    input.and_then(|input| match classify(js::get(input, key)) {
        Kind::Number(value)
            if value.is_finite()
                && value.fract() == 0.0
                && value >= 0.0
                && value <= f64::from(u32::MAX) =>
        {
            Some(value as u32)
        }
        _ => None,
    })
}

fn response(result: Result<Snapshot, String>) -> ToolResponse {
    match result {
        Ok(snapshot) => ToolResponse::text(
            serde_json::to_string(&snapshot).expect("cube snapshot is JSON"),
            false,
        ),
        Err(message) => ToolResponse::text(message, true),
    }
}

fn schema(properties: Properties, required: Vec<&'static str>) -> Schema {
    Schema {
        r#type: "object",
        properties,
        required,
        additional_properties: false,
    }
}

pub fn App() -> JSX::Element {
    let (snapshot, set_snapshot) = use_state(|| Session::default().snapshot());
    let (presentation, set_presentation) = use_state(|| Presentation {
        id: 0,
        replay: false,
        snapshot: Session::default().snapshot(),
        turns: None,
    });
    let (celebration, set_celebration) = use_state(|| None::<u32>);
    let (storage, set_storage) = use_state(|| "Loading saved session…");
    let session = use_ref(Rc::new(LocalSession::new(
        set_snapshot,
        set_storage,
        set_presentation,
        set_celebration,
    )))
    .current();
    let (view, set_view) = use_state(|| {
        catch_unwind(|| {
            window
                .local_storage()
                .ok()
                .and_then(|storage| storage.get_item("agent-cube.camera"))
        })
        .ok()
        .flatten()
        .filter(|view| view == "front")
        .map_or("perspective", |_| "front")
    });
    let (camera_reset, set_camera_reset) = use_state(|| 0_u32);
    let (snap_view, set_snap_view) = use_state(|| false);
    let (problem, set_problem) = use_state(|| None::<String>);
    let (replaying, set_replaying) = use_state(|| false);
    let (sharing, set_sharing) = use_state(|| "Copy challenge link");
    use_effect(
        move || set_sharing.set("Copy challenge link"),
        Some(&(snapshot.facelets.clone(),)),
    );
    use_effect(
        move || {
            celebration.map(|_| {
                let timer =
                    js::set_timeout(Box::new(move || set_celebration.set(None)), Some(2800.0));
                move || js::clear_timeout(Some(timer))
            })
        },
        Some(&(*celebration,)),
    );
    let change_view = move |next| {
        let _ = catch_unwind(|| {
            if let Ok(storage) = window.local_storage() {
                let _ = storage.set_item("agent-cube.camera", next);
            }
        });
        flush_sync(move || {
            set_view.set(next);
            set_camera_reset.update(|value| value.wrapping_add(1));
        });
    };

    let initializing = Rc::clone(&session);
    use_effect(
        move || {
            let session = Rc::clone(&initializing);
            spawn(Box::new(async move {
                if let Err(error) = session.initialize().await {
                    set_storage.set("Recovery blocked");
                    set_problem.set(Some(error));
                }
            }));
            let incoming = Rc::clone(&initializing);
            let channel = catch_unwind(|| broadcast_channel::new("agent-cube.practice")).ok();
            if let Some(channel) = channel {
                channel.add_event_listener_named(
                    "message",
                    move |_| {
                        let session = Rc::clone(&incoming);
                        spawn(Box::new(async move {
                            if let Err(error) = session.read().await {
                                set_problem.set(Some(error));
                            }
                        }));
                    },
                    None,
                );
            }
            initializing.set_channel(channel);
            let focused = Rc::clone(&initializing);
            let focus = webapi::listener(move |_: &Event| {
                let session = Rc::clone(&focused);
                spawn(Box::new(async move {
                    if session.ready()
                        && let Err(error) = session.read().await
                    {
                        set_problem.set(Some(error));
                    }
                }));
            });
            window.add_event_listener(events::Focus, focus, None);
            let linking = Rc::clone(&initializing);
            let hash = webapi::listener(move |_: &HashChangeEvent| {
                let session = Rc::clone(&linking);
                spawn(Box::new(async move {
                    set_problem.set(session.load_link().await.err());
                }));
            });
            window.add_event_listener(events::Hashchange, hash, None);
            let cleanup = Rc::clone(&initializing);
            Some(move || {
                window.remove_event_listener(events::Focus, focus, None);
                window.remove_event_listener(events::Hashchange, hash, None);
                cleanup.set_channel(None);
                if let Some(channel) = channel {
                    channel.close();
                }
            })
        },
        Some(&()),
    );

    let read_session = Rc::clone(&session);
    let state_tool = use_web_mcp(ToolOptions {
        name: "get_cube_state",
        description: "Read all 54 stickers, solved status, move count and revision. Facelets are row-major URFDLB, each face viewed from outside. U white, R red, F green, D yellow, L orange, B blue. Camera changes never change notation. Use legal face moves to solve; shuffle history is not exposed.",
        input_schema: schema(Properties::default(), vec![]),
        annotations: Annotations {
            read_only_hint: true,
        },
        execute: move |input: Option<&Unknown>, options: ExecutionOptions| {
            let read_session = Rc::clone(&read_session);
            promise(async move {
                if options.signal.aborted() {
                    ToolResponse::text("Request cancelled.", true)
                } else if valid_keys(input, &[], &[]) {
                    response(read_session.read().await)
                } else {
                    ToolResponse::text("Expected an empty object: {}.", true)
                }
            })
        },
    });
    let move_session = Rc::clone(&session);
    let move_tool = use_web_mcp(ToolOptions {
        name: "apply_moves",
        description: "Apply 1–128 legal moves atomically to solve the cube. Notation: U D L R F B clockwise viewed from outside, ' for inverse, 2 for half turn; separate moves with spaces. Pass expectedRevision from get_cube_state and a unique commandId (1–128 ASCII characters). Retry the exact request with the same ID after a lost response; a retry returns its original receipt without repeating moves. Receipt revision can precede the current state; read again to plan new moves. Invalid or stale commands change nothing. Returns the committed state after the visible cube updates.",
        input_schema: schema(
            Properties {
                command_id: Some(TextProperty {
                    r#type: "string",
                    description: "Unique command ID; reuse only for an identical retry",
                    max_length: 128,
                }),
                moves: Some(TextProperty {
                    r#type: "string",
                    description: "Space-separated Singmaster moves, e.g. R U R' U'",
                    max_length: 1024,
                }),
                expected_revision: Some(IntegerProperty {
                    r#type: "integer",
                    minimum: 0,
                    maximum: u32::MAX,
                }),
                ..Properties::default()
            },
            vec!["moves", "expectedRevision", "commandId"],
        ),
        annotations: Annotations {
            read_only_hint: false,
        },
        execute: move |input: Option<&Unknown>, options: ExecutionOptions| {
            let move_session = Rc::clone(&move_session);
            promise(async move {
                if options.signal.aborted() {
                    ToolResponse::text("Request cancelled.", true)
                } else if valid_keys(input, &["moves", "expectedRevision", "commandId"], &[]) {
                    match (
                        classify(js::get(input, "moves")),
                        integer(input, "expectedRevision"),
                        classify(js::get(input, "commandId")),
                    ) {
                        (Kind::String(moves), Some(revision), Kind::String(id))
                            if moves.chars().count() <= 1024 =>
                        {
                            response(
                                move_session
                                    .submit(
                                        Command {
                                            command_id: id.into(),
                                            expected_revision: revision,
                                            action: Action::Moves(moves.into()),
                                        },
                                        Some(options.signal),
                                    )
                                    .await,
                            )
                        }
                        _ => ToolResponse::text(
                            "Expected moves as a string and expectedRevision as a nonnegative integer.",
                            true,
                        ),
                    }
                } else {
                    ToolResponse::text(
                        "Expected exactly moves, expectedRevision and commandId. Read the state first.",
                        true,
                    )
                }
            })
        },
    });
    let shuffle_session = Rc::clone(&session);
    let shuffle_tool = use_web_mcp(ToolOptions {
        name: "shuffle_cube",
        description: "Start a new local challenge from the solved cube with a legal random-move shuffle. Supply a unique commandId and expectedRevision from get_cube_state. Retry identical requests with the same ID to recover the original result. Optional length is 1–100, default 20. Clears the move count, advances the revision, and returns all stickers. The scramble sequence and seed are not exposed. This is not uniform random-state sampling.",
        input_schema: schema(
            Properties {
                command_id: Some(TextProperty {
                    r#type: "string",
                    description: "Unique command ID; reuse only for an identical retry",
                    max_length: 128,
                }),
                expected_revision: Some(IntegerProperty {
                    r#type: "integer",
                    minimum: 0,
                    maximum: u32::MAX,
                }),
                length: Some(IntegerProperty {
                    r#type: "integer",
                    minimum: 1,
                    maximum: 100,
                }),
                ..Properties::default()
            },
            vec!["commandId", "expectedRevision"],
        ),
        annotations: Annotations {
            read_only_hint: false,
        },
        execute: move |input: Option<&Unknown>, options: ExecutionOptions| {
            let shuffle_session = Rc::clone(&shuffle_session);
            promise(async move {
                if options.signal.aborted() {
                    ToolResponse::text("Request cancelled.", true)
                } else if valid_keys(input, &["commandId", "expectedRevision"], &["length"]) {
                    let length = if input.is_some_and(|value| object::has_own(value, "length")) {
                        integer(input, "length")
                    } else {
                        Some(20)
                    };
                    match (
                        length,
                        integer(input, "expectedRevision"),
                        classify(js::get(input, "commandId")),
                    ) {
                        (Some(length), Some(revision), Kind::String(id)) => response(
                            shuffle_session
                                .submit(
                                    Command {
                                        command_id: id.into(),
                                        expected_revision: revision,
                                        action: Action::Shuffle(length as usize),
                                    },
                                    Some(options.signal),
                                )
                                .await,
                        ),
                        _ => ToolResponse::text(
                            "Shuffle length must be an integer from 1 to 100.",
                            true,
                        ),
                    }
                } else {
                    ToolResponse::text(
                        "Expected commandId, expectedRevision and optional length.",
                        true,
                    )
                }
            })
        },
    });
    let view_tool = use_web_mcp(ToolOptions {
        name: "set_camera_view",
        description: "Show the cube from front or perspective. Only changes the camera; use get_cube_state to inspect hidden faces.",
        input_schema: schema(
            Properties {
                view: Some(ViewProperty {
                    r#type: "string",
                    r#enum: vec!["front", "perspective"],
                }),
                ..Properties::default()
            },
            vec!["view"],
        ),
        annotations: Annotations {
            read_only_hint: false,
        },
        execute: move |input: Option<&Unknown>, options: ExecutionOptions| {
            promise(async move {
                if options.signal.aborted() {
                    ToolResponse::text("Request cancelled.", true)
                } else if valid_keys(input, &["view"], &[]) {
                    match classify(js::get(input, "view")) {
                        Kind::String("front") => {
                            change_view("front");
                            ToolResponse::text("Camera view: front.", false)
                        }
                        Kind::String("perspective") => {
                            change_view("perspective");
                            ToolResponse::text("Camera view: perspective.", false)
                        }
                        _ => ToolResponse::text("View must be front or perspective.", true),
                    }
                } else {
                    ToolResponse::text("Expected exactly one view property.", true)
                }
            })
        },
    });
    let tools = [state_tool, move_tool, shuffle_tool, view_tool];
    let agent_status = if !session.ready() {
        "Session not ready"
    } else if tools.iter().any(|tool| tool.error.is_some()) {
        "Registration failed"
    } else if tools.iter().all(|tool| tool.registered) {
        "Ready · 4 tools"
    } else if tools.iter().all(|tool| tool.supported) {
        "Connecting…"
    } else {
        "Not available"
    };
    let shuffle_button = Rc::clone(&session);
    let sharing_session = Rc::clone(&session);
    let replay_session = Rc::clone(&session);
    let ready = session.ready();
    let finishing = Rc::clone(&session);
    let complete: Rc<dyn Fn(u32)> =
        Rc::new(move |revision| finishing.presentations.finish(revision));
    let detaching = Rc::clone(&session);
    let detach: Rc<dyn Fn()> = Rc::new(move || detaching.presentations.interrupt());
    let pace = Rc::clone(&session.presentations.pace);
    let colors: Vec<char> = snapshot.facelets.chars().collect();

    jsx! {
        <div className="app-shell">
            {celebration.filter(|revision| *revision == snapshot.revision && snapshot.solved).map(|revision| jsx! {
                <div className="confetti" key={revision} aria-hidden="true" data-revision={revision.to_string()}>
                    {(0_u32..48).map(|piece| { let (hex, _) = color(FACES[(piece % 6) as usize]); jsx! {
                        <span key={piece} style={CSSProperties::new()
                            .set("left", format!("{}%", piece * 37 % 100))
                            .set("--color", format!("#{hex:06x}"))
                            .set("--sway", format!("{}px", (piece * 53 % 241) as i32 - 120))
                            .set("--spin", format!("{}deg", 180 + piece * 97 % 900))
                            .set("animationDelay", format!("{}ms", piece % 8 * 25))} />
                    } }).collect::<Vec<_>>()}
                </div>
            })}
            <header className="topbar">
                <a className="brand" href="./" aria-label="AgentCube home"><span className="brand-icon" aria-hidden="true">{"◇"}</span>{"AgentCube"}</a>
                <span className="mode-badge"><span className="status-dot" />{"LOCAL PLAY"}</span>
            </header>
            <main>
                <section className="intro" aria-labelledby="title">
                    <p className="eyebrow">{"HUMAN CURIOSITY. AGENT POSSIBILITY."}</p>
                    <h1 id="title">{"A little twist."}<br /><span>{"A challenge worth solving."}</span></h1>
                    <p className="description">{"Shuffle it. Study it. Find your way back."}<br />{"You and your agent play by the same rules."}</p>
                </section>
                <section className="lab" aria-label="Cube playground">
                    <div className="stage">
                        <div className="stage-top"><span className="eyebrow">{"01 / YOUR NEXT CHALLENGE"}</span><span className="preview-label">{"3 × 3 · LIVE"}</span></div>
                        <Preview view={*view} step={presentation.clone()} complete={complete} detach={detach} pace={pace} cameraReset={*camera_reset} snap={*snap_view} />
                        <div className="stage-bottom"><div className="view-controls" role="group" aria-label="Drag mode"><button type="button" aria-pressed={!*snap_view} onClick={move |_| set_snap_view.set(false)}>{"Free"}</button><button type="button" aria-pressed={*snap_view} onClick={move |_| set_snap_view.set(true)}>{"Snap"}</button></div><div className="view-controls" role="group" aria-label="Camera view">
                            <button type="button" aria-pressed={*view == "perspective"} onClick={move |_| change_view("perspective")}>{"Perspective"}</button>
                            <button type="button" aria-pressed={*view == "front"} onClick={move |_| change_view("front")}>{"Front"}</button>
                        </div></div>
                    </div>
                    <aside className="lab-panel" aria-label="Game controls">
                        <p className="eyebrow">{"ONE CUBE. EVERY POSSIBILITY."}</p>
                        <h2>{"Your move."}</h2>
                        <div className="game-status" role="status" data-testid="cube-status">{if snapshot.solved { "Solved" } else { "In progress" }}<span>{format!("{} {}", snapshot.move_count, if snapshot.move_count == 1 { "move" } else { "moves" })}</span></div>
                        <button className="shuffle-button" type="button" disabled={!ready} onClick={move |_| { let session = Rc::clone(&shuffle_button); spawn(Box::new(async move { set_problem.set(session.play(Action::Shuffle(20)).await.err()); })); }}>{"Shuffle cube ↗"}</button>
                        <div className="move-controls" role="group" aria-label="Face turns">
                            {FACES.into_iter().map(|face| {
                                jsx! { <div className="move-column" key={face.to_string()}>
                                    {["", "'", "2"].into_iter().map(|suffix| {
                                        let movement = format!("{face}{suffix}");
                                        let command = movement.clone();
                                        let session = Rc::clone(&session);
                                        jsx! { <button type="button" key={movement.clone()} aria-label={format!("Turn {movement}")} disabled={!ready} onClick={move |_| {
                                            let session = Rc::clone(&session); let command = command.clone();
                                            spawn(Box::new(async move { set_problem.set(session.play(Action::Moves(command)).await.err()); }));
                                        }}>{movement}</button> }
                                    }).collect::<Vec<_>>()}
                                </div> }
                            }).collect::<Vec<_>>()}
                        </div>
                        <p className="notation-note">{"Clockwise from outside · ' inverse · 2 half turn"}</p>
                        {(snapshot.solved && snapshot.move_count > 0).then(|| jsx! { <button className="replay-button" type="button" disabled={!ready || snapshot.move_count == 0 || *replaying} title="Watch this challenge from its starting position" onClick={move |_| {
                            let session = Rc::clone(&replay_session);
                            set_replaying.set(true);
                            spawn(Box::new(async move {
                                set_problem.set(session.replay().await.err());
                                set_replaying.set(false);
                            }));
                        }}>{if *replaying { "Replaying…".to_string() } else if snapshot.move_count == 0 { "Replay moves".to_string() } else { format!("Replay {} {}", snapshot.move_count, if snapshot.move_count == 1 { "move" } else { "moves" }) }}</button> })}
                        {problem.as_ref().map(|message| jsx! { <p role="alert">{message}</p> })}
                        <dl className="status-list">
                            <div><dt>{"Game backend"}</dt><dd>{"Not required"}</dd></div>
                            <div><dt>{"Browser storage"}</dt><dd data-testid="storage-status">{*storage}</dd></div>
                            <div><dt>{"WebMCP"}</dt><dd data-testid="agent-status">{agent_status}</dd></div>
                        </dl>
                        <div className="agent-note"><span className="eyebrow">{"INVITE AN AGENT"}</span><p>{"“Shuffle the cube, inspect its faces, then solve it using legal moves.”"}</p><small>{"Requires WebMCP and a compatible agent. Your agent chooses the solution; the app checks every move."}</small></div>
                        <button className="share-button" type="button" disabled={!ready} onClick={move |_| {
                            let session = Rc::clone(&sharing_session);
                            spawn(Box::new(async move {
                                match session.read().await {
                                    Ok(_) => { let copied = settle(window.navigator().clipboard().write_text(&window.location().href())).await.is_ok(); set_sharing.set(if copied { "Link copied" } else { "Copy the address bar to share" }); },
                                    Err(error) => set_problem.set(Some(error)),
                                }
                            }));
                        }}>{*sharing}</button>
                        <p className="scope-note">{"Share the address to challenge someone with this exact position and zero moves. Saved play resumes on reload when storage is available. Multiplayer is coming next."}</p>
                    </aside>
                </section>
                <section className="cube-net" aria-label="All six cube faces" data-revision={snapshot.revision.to_string()}>
                    {FACES.into_iter().enumerate().map(|(face_index, face)| {
                        let face_colors = colors[face_index * 9..face_index * 9 + 9].to_vec();
                        jsx! { <div className={format!("net-face net-{face}")} key={face.to_string()}>
                            <span className="face-label">{face.to_string()}</span>
                            <div className="face-grid" role="img" aria-label={format!("{face} face")}>
                                {face_colors.into_iter().map(|value| { let (hex, name) = color(value); jsx! { <span data-color={value.to_string()} title={name} style={CSSProperties::new().set("backgroundColor", format!("#{hex:06x}"))} /> } }).collect::<Vec<_>>()}
                            </div>
                        </div> }
                    }).collect::<Vec<_>>()}
                    <p className="net-caption">{"ALL SIX FACES · U WHITE / R RED / F GREEN / D YELLOW / L ORANGE / B BLUE"}</p>
                </section>
            </main>
            <footer><span>{"RUST IN. POSSIBILITY OUT."}</span><a href="https://github.com/rust-js-lang/agent-cube">{"Follow the build ↗"}</a></footer>
        </div>
    }
}
