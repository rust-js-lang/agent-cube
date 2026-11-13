use std::cell::{Cell, RefCell};
use std::f64::consts::PI;
use std::rc::Rc;

use cube_core::{Cube, FACES, Move, NORMALS, facelet_position};
use react::{CSSProperties, JSX, jsx, use_effect, use_ref, use_state};
use webapi::{Event, EventTargetExt, HTMLCanvasElement, document, events, window};

use crate::colors::color;
use crate::presentation::Presentation;
use crate::three::{
    self, Geometry, Material, MaterialOptions, Node, PhongOptions, RendererOptions,
};

pub struct PreviewProps {
    pub view: &'static str,
    pub step: Presentation,
    pub complete: Rc<dyn Fn(u32)>,
    pub detach: Rc<dyn Fn()>,
}

struct Cubie {
    position: [i8; 3],
    group: &'static three::Group,
}

struct Motion {
    cube: Cube,
    moves: Vec<Move>,
    index: usize,
    started: Option<f64>,
    elapsed: f64,
    unit_duration: f64,
    prepared: bool,
    target: String,
    revision: u32,
    presentation_id: u32,
    complete: Rc<dyn Fn(u32)>,
}

struct Drawing {
    scene: &'static three::Scene,
    layer: &'static three::Group,
    cubies: Vec<Cubie>,
    camera: &'static three::PerspectiveCamera,
    renderer: &'static three::WebGLRenderer,
    materials: Vec<&'static three::MeshBasicMaterial>,
    canvas: &'static HTMLCanvasElement,
    motion: RefCell<Option<Motion>>,
    frame: Cell<Option<f64>>,
    deadline: Cell<Option<&'static js::TimeoutId>>,
    completed_turns: Cell<u32>,
}

fn reduced_motion() -> bool {
    window
        .match_media("(prefers-reduced-motion: reduce)")
        .matches()
}
fn weight(movement: Move) -> f64 {
    if movement.rotation().1.abs() == 2 {
        1.5
    } else {
        1.0
    }
}

impl Drawing {
    fn draw(&self) {
        self.renderer.render(self.scene, self.camera);
    }

    fn resize(&self) {
        let width = self.canvas.client_width().max(1.0);
        let height = self.canvas.client_height().max(1.0);
        self.renderer
            .set_pixel_ratio(window.device_pixel_ratio().min(2.0));
        self.renderer.set_size(width, height, false);
        self.camera.set_aspect(width / height);
        self.camera.update_projection_matrix();
        self.draw();
    }

    fn camera_view(&self, view: &str) {
        if view == "front" {
            self.camera.position().set(0.0, 0.0, 9.5);
        } else {
            self.camera.position().set(6.2, 5.0, 7.0);
        }
        self.camera.look_at(0.0, 0.0, 0.0);
        self.draw();
    }

    fn paint(&self, facelets: &str) {
        for (material, value) in self.materials.iter().zip(facelets.chars()) {
            material.color().set_hex(color(value).0);
        }
        self.canvas
            .set_attribute("data-presented-facelets", facelets);
    }

    fn reset_layer(&self) {
        for cubie in &self.cubies {
            self.scene.add(cubie.group);
        }
        self.layer.rotation().set(0.0, 0.0, 0.0);
    }

    fn prepare_layer(&self, movement: Move) {
        let normal = movement.rotation().0;
        for cubie in &self.cubies {
            if cubie
                .position
                .iter()
                .zip(normal)
                .map(|(position, normal)| position * normal)
                .sum::<i8>()
                == 1
            {
                self.layer.add(cubie.group);
            }
        }
        self.canvas.set_attribute("data-turn", movement.notation());
    }

    fn rotate_layer(&self, movement: Move, progress: f64) {
        let (normal, turns) = movement.rotation();
        let eased = progress * progress * (3.0 - 2.0 * progress);
        let angle = -f64::from(turns) * PI / 2.0 * eased;
        self.layer.rotation().set(
            f64::from(normal[0]) * angle,
            f64::from(normal[1]) * angle,
            f64::from(normal[2]) * angle,
        );
        self.canvas
            .set_attribute("data-turn-angle", angle.to_string());
    }

    fn cancel_callbacks(&self) {
        if let Some(frame) = self.frame.take() {
            window.cancel_animation_frame(frame);
        }
        js::clear_timeout(self.deadline.take());
    }

    fn finish_step(&self, motion: Motion) {
        self.cancel_callbacks();
        self.reset_layer();
        self.paint(&motion.target);
        self.canvas.set_attribute("data-motion", "idle");
        self.canvas.set_attribute("data-turn", "");
        self.canvas.set_attribute("data-turn-angle", "0");
        self.canvas
            .set_attribute("data-visual-revision", motion.revision.to_string());
        self.draw();
        self.canvas.set_attribute("data-playback", "live");
        (motion.complete)(motion.presentation_id);
    }

    fn finish_now(&self) {
        let motion = self.motion.borrow_mut().take();
        if let Some(motion) = motion {
            self.finish_step(motion);
        }
    }

    fn schedule(self: &Rc<Self>) {
        let drawing = Rc::clone(self);
        self.frame.set(Some(
            window.request_animation_frame(Box::new(move |time| drawing.tick(time))),
        ));
    }

    fn tick(self: &Rc<Self>, time: f64) {
        self.frame.set(None);
        let Some(mut motion) = self.motion.borrow_mut().take() else {
            return;
        };
        if document.hidden() || reduced_motion() {
            self.finish_step(motion);
            return;
        }
        let started = *motion.started.get_or_insert(time);
        let elapsed = (time - started).max(0.0);
        while let Some(movement) = motion.moves.get(motion.index).copied() {
            let duration = motion.unit_duration * weight(movement);
            if elapsed < motion.elapsed + duration {
                if !motion.prepared {
                    self.prepare_layer(movement);
                    motion.prepared = true;
                }
                self.rotate_layer(movement, (elapsed - motion.elapsed) / duration);
                break;
            }
            self.reset_layer();
            motion.cube.apply(movement);
            self.paint(&motion.cube.facelets());
            self.completed_turns.set(self.completed_turns.get() + 1);
            self.canvas.set_attribute(
                "data-completed-turns",
                self.completed_turns.get().to_string(),
            );
            motion.index += 1;
            motion.elapsed += duration;
            motion.prepared = false;
        }
        if motion.index == motion.moves.len() {
            self.finish_step(motion);
        } else {
            *self.motion.borrow_mut() = Some(motion);
            self.draw();
            self.schedule();
        }
    }

    fn begin(self: &Rc<Self>, step: Presentation, complete: Rc<dyn Fn(u32)>) {
        self.finish_now();
        self.canvas
            .set_attribute("data-playback", if step.replay { "replay" } else { "live" });
        let turns = step
            .turns
            .filter(|_| !reduced_motion() && !document.hidden());
        if let Some(turns) = turns
            && let Ok(cube) = Cube::from_facelets(&turns.from)
        {
            self.paint(&turns.from);
            let units: f64 = turns.moves.iter().map(|movement| weight(*movement)).sum();
            self.canvas.set_attribute("data-motion", "turning");
            let limit = if step.replay { 60_000.0 } else { 2400.0 };
            let duration = (units * 240.0).min(limit);
            let deadline_ms = if step.replay {
                duration.max(2400.0) + 200.0
            } else {
                2600.0
            };
            *self.motion.borrow_mut() = Some(Motion {
                cube,
                moves: turns.moves,
                index: 0,
                started: None,
                elapsed: 0.0,
                unit_duration: 240.0_f64.min(limit / units.max(1.0)),
                prepared: false,
                target: step.snapshot.facelets,
                revision: step.snapshot.revision,
                presentation_id: step.id,
                complete,
            });
            self.draw();
            let deadline = Rc::clone(self);
            self.deadline.set(Some(js::set_timeout(
                Box::new(move || deadline.finish_now()),
                Some(deadline_ms),
            )));
            self.schedule();
        } else {
            self.paint(&step.snapshot.facelets);
            self.canvas.set_attribute("data-playback", "live");
            self.canvas.set_attribute("data-motion", "idle");
            self.canvas
                .set_attribute("data-visual-revision", step.snapshot.revision.to_string());
            self.draw();
            window.queue_microtask(Box::new(move || complete(step.id)));
        }
    }
}

pub fn Preview(
    PreviewProps {
        view,
        step,
        complete,
        detach,
    }: PreviewProps,
) -> JSX::Element {
    let canvas = use_ref(None::<&'static HTMLCanvasElement>);
    let drawing = use_ref(None::<Rc<Drawing>>);
    let (problem, set_problem) = use_state(|| None::<&'static str>);
    let detaching = Rc::clone(&detach);

    // Renderer lifetime belongs to the canvas, not camera or cube updates.
    use_effect(
        move || -> Option<Box<dyn FnOnce()>> {
            let detach = Rc::clone(&detaching);
            let canvas = canvas.current().expect("mounted canvas");
            match three::renderer(RendererOptions {
                canvas,
                antialias: true,
                alpha: true,
            }) {
                Ok(renderer) => {
                    let scene = three::scene();
                    let camera = three::perspective_camera(35.0, 1.0, 0.1, 100.0);
                    let layer = three::group();
                    scene.add(layer);
                    scene.add(three::ambient_light(0xffffff, 2.0));
                    let key = three::directional_light(0xffffff, 3.0);
                    key.position().set(4.0, 7.0, 5.0);
                    scene.add(key);
                    let body_geometry = three::rounded_box(0.95, 0.95, 0.95, 1, 0.055);
                    let black = three::phong_material(PhongOptions {
                        color: 0x19201b,
                        shininess: 55.0,
                        specular: 0x647265,
                    });
                    let mut cubies = Vec::new();
                    for x in -1_i8..=1 {
                        for y in -1_i8..=1 {
                            for z in -1_i8..=1 {
                                let group = three::group();
                                group
                                    .position()
                                    .set(f64::from(x), f64::from(y), f64::from(z));
                                group.add(three::mesh(body_geometry, black));
                                scene.add(group);
                                cubies.push(Cubie {
                                    position: [x, y, z],
                                    group,
                                });
                            }
                        }
                    }
                    let geometry = three::rounded_box(0.86, 0.86, 0.06, 1, 0.03);
                    let mut materials = Vec::with_capacity(54);
                    for (face, normal) in NORMALS.iter().enumerate() {
                        for cell in 0..9 {
                            let material = three::material(MaterialOptions {
                                color: color(FACES[face]).0,
                            });
                            let sticker = three::mesh(geometry, material);
                            let position = facelet_position(face, cell);
                            sticker.position().set(
                                f64::from(normal[0]) * 0.49,
                                f64::from(normal[1]) * 0.49,
                                f64::from(normal[2]) * 0.49,
                            );
                            let [x, y, z] = match face {
                                0 => [-PI / 2.0, 0.0, 0.0],
                                1 => [0.0, PI / 2.0, 0.0],
                                3 => [PI / 2.0, 0.0, 0.0],
                                4 => [0.0, -PI / 2.0, 0.0],
                                5 => [0.0, PI, 0.0],
                                _ => [0.0, 0.0, 0.0],
                            };
                            sticker.rotation().set(x, y, z);
                            cubies
                                .iter()
                                .find(|cubie| cubie.position == position)
                                .expect("sticker cubie")
                                .group
                                .add(sticker);
                            materials.push(material);
                        }
                    }
                    let objects = Rc::new(Drawing {
                        scene,
                        layer,
                        cubies,
                        camera,
                        renderer,
                        materials,
                        canvas,
                        motion: RefCell::new(None),
                        frame: Cell::new(None),
                        deadline: Cell::new(None),
                        completed_turns: Cell::new(0),
                    });
                    objects.camera_view("perspective");
                    objects.resize();
                    drawing.set_current(Some(Rc::clone(&objects)));
                    let resized = Rc::clone(&objects);
                    let resize = webapi::listener(move |_: &Event| resized.resize());
                    let losing = Rc::clone(&objects);
                    let lost = webapi::listener(move |event: &Event| {
                        event.prevent_default();
                        set_problem.set(Some("Compatible view · graphics context lost"));
                        losing.finish_now();
                    });
                    let hiding = Rc::clone(&objects);
                    let hidden = webapi::listener(move |_: &Event| {
                        if document.hidden() {
                            hiding.finish_now();
                        }
                    });
                    let reducing = Rc::clone(&objects);
                    let reduced = webapi::listener(move |_: &Event| {
                        if reduced_motion() {
                            reducing.finish_now();
                        }
                    });
                    let preference = window.match_media("(prefers-reduced-motion: reduce)");
                    window.add_event_listener(events::Resize, resize, None);
                    document.add_event_listener_named("visibilitychange", hidden, None);
                    preference.add_event_listener_named("change", reduced, None);
                    canvas.add_event_listener_named("webglcontextlost", lost, None);
                    Some(Box::new(move || {
                        window.remove_event_listener(events::Resize, resize, None);
                        document.remove_event_listener_named("visibilitychange", hidden, None);
                        preference.remove_event_listener_named("change", reduced, None);
                        canvas.remove_event_listener_named("webglcontextlost", lost, None);
                        objects.cancel_callbacks();
                        drawing.set_current(None);
                        detach();
                        for material in &objects.materials {
                            material.dispose();
                        }
                        black.dispose();
                        body_geometry.dispose();
                        geometry.dispose();
                        renderer.dispose();
                        // Do not force context loss: React may remount on this same canvas.
                    }))
                }
                Err(_) => {
                    set_problem.set(Some("Compatible view · WebGL unavailable"));
                    Some(Box::new(move || detach()))
                }
            }
        },
        Some(&()),
    );

    use_effect(
        move || {
            if let Some(drawing) = drawing.current() {
                drawing.camera_view(view);
            }
        },
        Some(&(view,)),
    );
    let animated = step.clone();
    use_effect(
        move || {
            let complete = Rc::clone(&complete);
            let step = animated.clone();
            if problem.is_none()
                && let Some(drawing) = drawing.current()
            {
                drawing.begin(step, complete);
            } else {
                if let Some(drawing) = drawing.current() {
                    drawing.finish_now();
                }
                if let Some(canvas) = canvas.current() {
                    canvas.set_attribute("data-presented-facelets", step.snapshot.facelets.clone());
                    canvas
                        .set_attribute("data-visual-revision", step.snapshot.revision.to_string());
                    canvas.set_attribute("data-motion", "idle");
                    canvas.set_attribute("data-playback", "live");
                }
                window.queue_microtask(Box::new(move || complete(step.snapshot.revision)));
            }
        },
        Some(&(step.id, *problem)),
    );

    let values: Vec<char> = step.snapshot.facelets.chars().collect();
    jsx! {
        <div className="cube-preview" data-view={view} data-renderer={if problem.is_some() { "compatible" } else { "webgl" }}>
            <canvas ref={canvas} style={CSSProperties::new().set("display", if problem.is_some() { "none" } else { "block" })} aria-label="Interactive cube rendering" role="img" />
            {problem.is_some().then(|| jsx! {
                <div className="compatible-stage" role="img" aria-label="Cube with compatible rendering">
                    <div className="compatible-cube">
                        {FACES.into_iter().enumerate().map(|(face, name)| {
                            let stickers = values[face * 9..face * 9 + 9].to_vec();
                            jsx! { <div className={format!("compatible-face face-{name}")} key={name.to_string()}>
                                {stickers.into_iter().map(|value| { let (hex, label) = color(value); jsx! { <span data-color={value.to_string()} title={label} style={CSSProperties::new().set("backgroundColor", format!("#{hex:06x}"))} /> } }).collect::<Vec<_>>()}
                            </div> }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            })}
            {problem.map(|message| jsx! { <small className="renderer-note" role="status">{message}</small> })}
        </div>
    }
}
