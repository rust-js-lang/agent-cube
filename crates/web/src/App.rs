use std::panic::catch_unwind;
use std::rc::Rc;

mod colors;
mod preview;
mod session;
mod three;
mod webmcp;

use cube_core::{FACES, Session, Snapshot};
use js::{Kind, Unknown, classify, object, settle, spawn};
use react::{CSSProperties, JSX, dom::flush_sync, jsx, use_effect, use_ref, use_state};
use webapi::window;

use colors::color;
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
    let session = use_ref(Rc::new(LocalSession::new(set_snapshot))).current();
    let (view, set_view) = use_state(|| "perspective");
    let (storage, set_storage) = use_state(|| "Checking…");
    let (problem, set_problem) = use_state(|| None::<String>);
    let change_view = move |next| flush_sync(move || set_view.set(next));

    use_effect(
        move || {
            spawn(Box::new(async move {
                // Availability only; persisted sessions are the next milestone.
                let available = match catch_unwind(|| window.indexed_db().databases()) {
                    Ok(request) => settle(request).await.is_ok(),
                    Err(_) => false,
                };
                set_storage.set(if available {
                    "Available"
                } else {
                    "Unavailable"
                });
            }));
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
            if options.signal.aborted() {
                ToolResponse::text("Request cancelled.", true)
            } else if valid_keys(input, &[], &[]) {
                response(Ok(read_session.snapshot()))
            } else {
                ToolResponse::text("Expected an empty object: {}.", true)
            }
        },
    });
    let move_session = Rc::clone(&session);
    let move_tool = use_web_mcp(ToolOptions {
        name: "apply_moves",
        description: "Apply 1–128 legal moves atomically to solve the cube. Notation: U D L R F B clockwise viewed from outside, ' for inverse, 2 for half turn; separate moves with spaces. Pass expectedRevision from get_cube_state. Invalid or stale commands change nothing. Returns the committed state after the visible cube updates.",
        input_schema: schema(
            Properties {
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
            vec!["moves", "expectedRevision"],
        ),
        annotations: Annotations {
            read_only_hint: false,
        },
        execute: move |input: Option<&Unknown>, options: ExecutionOptions| {
            if options.signal.aborted() {
                ToolResponse::text("Request cancelled.", true)
            } else if valid_keys(input, &["moves", "expectedRevision"], &[]) {
                match (
                    classify(js::get(input, "moves")),
                    integer(input, "expectedRevision"),
                ) {
                    (Kind::String(moves), Some(revision)) if moves.chars().count() <= 1024 => {
                        response(move_session.apply(moves, revision))
                    }
                    _ => ToolResponse::text(
                        "Expected moves as a string and expectedRevision as a nonnegative integer.",
                        true,
                    ),
                }
            } else {
                ToolResponse::text(
                    "Expected exactly moves and expectedRevision. Read the state first.",
                    true,
                )
            }
        },
    });
    let shuffle_session = Rc::clone(&session);
    let shuffle_tool = use_web_mcp(ToolOptions {
        name: "shuffle_cube",
        description: "Start a new local challenge from the solved cube with a legal random-move shuffle. Optional length is 1–100, default 20. Clears the move count, advances the revision, and returns all stickers. The scramble sequence and seed are not exposed. This is not uniform random-state sampling.",
        input_schema: schema(
            Properties {
                length: Some(IntegerProperty {
                    r#type: "integer",
                    minimum: 1,
                    maximum: 100,
                }),
                ..Properties::default()
            },
            vec![],
        ),
        annotations: Annotations {
            read_only_hint: false,
        },
        execute: move |input: Option<&Unknown>, options: ExecutionOptions| {
            if options.signal.aborted() {
                ToolResponse::text("Request cancelled.", true)
            } else if valid_keys(input, &[], &["length"]) {
                let length = if input.is_some_and(|value| object::has_own(value, "length")) {
                    integer(input, "length")
                } else {
                    Some(20)
                };
                match length {
                    Some(length) => response(shuffle_session.shuffle(length as usize)),
                    None => {
                        ToolResponse::text("Shuffle length must be an integer from 1 to 100.", true)
                    }
                }
            } else {
                ToolResponse::text("Expected {} or {\"length\":20}.", true)
            }
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
        },
    });
    let tools = [state_tool, move_tool, shuffle_tool, view_tool];
    let agent_status = if tools.iter().any(|tool| tool.error.is_some()) {
        "Registration failed"
    } else if tools.iter().all(|tool| tool.registered) {
        "Ready · 4 tools"
    } else if tools.iter().all(|tool| tool.supported) {
        "Connecting…"
    } else {
        "Not available"
    };
    let shuffle_button = Rc::clone(&session);
    let colors: Vec<char> = snapshot.facelets.chars().collect();

    jsx! {
        <div className="app-shell">
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
                        <Preview view={*view} facelets={snapshot.facelets.clone()} />
                        <div className="stage-bottom"><span>{"LOOK AROUND"}</span><div className="view-controls" role="group" aria-label="Camera view">
                            <button type="button" aria-pressed={*view == "perspective"} onClick={move |_| change_view("perspective")}>{"Perspective"}</button>
                            <button type="button" aria-pressed={*view == "front"} onClick={move |_| change_view("front")}>{"Front"}</button>
                        </div></div>
                    </div>
                    <aside className="lab-panel" aria-label="Game controls">
                        <p className="eyebrow">{"ONE CUBE. EVERY POSSIBILITY."}</p>
                        <h2>{"Your move."}</h2>
                        <div className="game-status" role="status" data-testid="cube-status">{if snapshot.solved { "Solved" } else { "In progress" }}<span>{format!("{} moves", snapshot.move_count)}</span></div>
                        <button className="shuffle-button" type="button" onClick={move |_| { set_problem.set(shuffle_button.shuffle(20).err()); }}>{"Shuffle cube ↗"}</button>
                        <div className="move-controls" role="group" aria-label="Face turns">
                            {FACES.into_iter().map(|face| {
                                jsx! { <div className="move-column" key={face.to_string()}>
                                    {["", "'", "2"].into_iter().map(|suffix| {
                                        let movement = format!("{face}{suffix}");
                                        let command = movement.clone();
                                        let session = Rc::clone(&session);
                                        jsx! { <button type="button" key={movement.clone()} aria-label={format!("Turn {movement}")} onClick={move |_| {
                                            set_problem.set(session.apply(&command, session.snapshot().revision).err());
                                        }}>{movement}</button> }
                                    }).collect::<Vec<_>>()}
                                </div> }
                            }).collect::<Vec<_>>()}
                        </div>
                        <p className="notation-note">{"Clockwise from outside · ' inverse · 2 half turn"}</p>
                        {problem.as_ref().map(|message| jsx! { <p role="alert">{message}</p> })}
                        <dl className="status-list">
                            <div><dt>{"Game backend"}</dt><dd>{"Not required"}</dd></div>
                            <div><dt>{"Browser storage"}</dt><dd data-testid="storage-status">{*storage}</dd></div>
                            <div><dt>{"WebMCP"}</dt><dd data-testid="agent-status">{agent_status}</dd></div>
                        </dl>
                        <div className="agent-note"><span className="eyebrow">{"INVITE AN AGENT"}</span><p>{"“Shuffle the cube, inspect its faces, then solve it using legal moves.”"}</p><small>{"Requires WebMCP and a compatible agent. Your agent chooses the solution; the app checks every move."}</small></div>
                        <p className="scope-note">{"Local session. Reload starts fresh. Saved games and multiplayer are coming next."}</p>
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
