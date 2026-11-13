use std::panic::catch_unwind;

mod preview;
mod three;
mod webmcp;

use js::{Kind, Unknown, classify, object, settle, spawn};
use react::{JSX, dom::flush_sync, jsx, use_effect, use_state};
use webapi::window;

use preview::Preview;
use webmcp::{
    Annotations, ExecutionOptions, Properties, Schema, ToolOptions, ToolResponse, ViewProperty,
    use_web_mcp,
};

fn camera_view(input: Option<&Unknown>) -> Option<&'static str> {
    match classify(input) {
        Kind::Object(_) => {
            let input = input.expect("object input");
            if object::keys(input).len() == 1 {
                match classify(js::get(input, "view")) {
                    Kind::String("front") => Some("front"),
                    Kind::String("perspective") => Some("perspective"),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn App() -> JSX::Element {
    let (view, set_view) = use_state(|| "perspective");
    let (storage, set_storage) = use_state(|| "Checking…");
    let change_view = move |next| flush_sync(move || set_view.set(next));

    use_effect(
        move || {
            spawn(Box::new(async move {
                // This milestone probes the browser boundary. Session persistence
                // and transactional writes arrive with the local referee.
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

    let state_tool = use_web_mcp(ToolOptions {
        name: "get_preview_state",
        description: "Read AgentCube's preview camera view. Cube moves are not implemented yet.",
        input_schema: Schema {
            r#type: "object",
            properties: Properties::default(),
            required: vec![],
            additional_properties: false,
        },
        annotations: Annotations {
            read_only_hint: true,
        },
        execute: move |input: Option<&Unknown>, options: ExecutionOptions| {
            if options.signal.aborted() {
                ToolResponse::text("Request cancelled.", true)
            } else if input.is_some_and(|input| {
                matches!(classify(Some(input)), Kind::Object(_)) && object::keys(input).is_empty()
            }) {
                ToolResponse::text(
                    format!(
                        "Camera view: {}. Preview only; cube moves are not implemented.",
                        *view
                    ),
                    false,
                )
            } else {
                ToolResponse::text("Expected an empty object: {}.", true)
            }
        },
    });
    let view_tool = use_web_mcp(ToolOptions {
        name: "set_camera_view",
        description: "Switch the visible cube preview to front or perspective. Changes the camera, not the cube's logical state.",
        input_schema: Schema {
            r#type: "object",
            properties: Properties {
                view: Some(ViewProperty {
                    r#type: "string",
                    r#enum: vec!["front", "perspective"],
                }),
            },
            required: vec!["view"],
            additional_properties: false,
        },
        annotations: Annotations {
            read_only_hint: false,
        },
        execute: move |input: Option<&Unknown>, options: ExecutionOptions| {
            if options.signal.aborted() {
                ToolResponse::text("Request cancelled.", true)
            } else if let Some(next) = camera_view(input) {
                change_view(next);
                ToolResponse::text(format!("Camera view: {next}."), false)
            } else {
                ToolResponse::text(
                    "Expected exactly {\"view\":\"front\"} or {\"view\":\"perspective\"}.",
                    true,
                )
            }
        },
    });
    let agent_status = if state_tool.error.is_some() || view_tool.error.is_some() {
        "Registration failed"
    } else if state_tool.registered && view_tool.registered {
        "Ready · 2 tools"
    } else if state_tool.supported && view_tool.supported {
        "Connecting…"
    } else {
        "Not available"
    };

    jsx! {
        <div className="app-shell">
            <header className="topbar">
                <a className="brand" href="./" aria-label="AgentCube home"><span className="brand-icon" aria-hidden="true">{"◇"}</span>{"AgentCube"}</a>
                <span className="mode-badge"><span className="status-dot" />{"LOCAL LAB"}</span>
            </header>
            <main>
                <section className="intro" aria-labelledby="title">
                    <p className="eyebrow">{"HUMAN CURIOSITY. AGENT POSSIBILITY."}</p>
                    <h1 id="title">{"A little twist."}<br /><span>{"A new kind of playground."}</span></h1>
                    <p className="description">{"A cube, a human, an agent. One shared world to explore."}<br />{"Built with Rust. At home in your browser."}</p>
                </section>
                <section className="lab" aria-label="Cube preview lab">
                    <div className="stage">
                        <div className="stage-top"><span className="eyebrow">{"01 / THE FIRST TURN"}</span><span className="preview-label">{"3 × 3 · PREVIEW"}</span></div>
                        <Preview view={*view} />
                        <div className="stage-bottom"><span>{"LOOK AROUND"}</span><div className="view-controls" role="group" aria-label="Camera view">
                            <button type="button" aria-pressed={*view == "perspective"} onClick={move |_| change_view("perspective")}>{"Perspective"}</button>
                            <button type="button" aria-pressed={*view == "front"} onClick={move |_| change_view("front")}>{"Front"}</button>
                        </div></div>
                    </div>
                    <aside className="lab-panel" aria-label="Lab status">
                        <p className="eyebrow">{"SMALL BEGINNINGS. REAL CONNECTIONS."}</p>
                        <h2>{"Meet your"}<br />{"next opponent."}</h2>
                        <p>{"For now, explore the cube from two angles. You and an agent use the same controls."}</p>
                        <dl className="status-list">
                            <div><dt>{"Game backend"}</dt><dd>{"Not required"}</dd></div>
                            <div><dt>{"Browser storage"}</dt><dd data-testid="storage-status">{*storage}</dd></div>
                            <div><dt>{"WebMCP"}</dt><dd data-testid="agent-status" role="status">{agent_status}</dd></div>
                        </dl>
                        <div className="agent-note"><span className="eyebrow">{"INVITE AN AGENT"}</span><p>{"Ask a WebMCP-compatible agent: “Show me the front of the cube.”"}</p><small>{"Requires a browser with WebMCP enabled and a compatible agent client."}</small></div>
                        <p className="scope-note">{"Bootstrap milestone. Cube moves, saved games, and multiplayer are coming next."}</p>
                    </aside>
                </section>
            </main>
            <footer><span>{"RUST IN. POSSIBILITY OUT."}</span><a href="https://github.com/rust-js-lang/agent-cube">{"Follow the build ↗"}</a></footer>
        </div>
    }
}
