use std::f64::consts::PI;
use std::rc::Rc;

use cube_core::{FACES, NORMALS, facelet_position};
use react::{CSSProperties, JSX, jsx, use_effect, use_ref, use_state};
use webapi::{Event, EventTargetExt, HTMLCanvasElement, events, window};

use crate::colors::color;
use crate::three::{self, MaterialOptions, RendererOptions};

pub struct PreviewProps {
    pub view: &'static str,
    pub facelets: String,
}

struct Drawing {
    scene: &'static three::Scene,
    camera: &'static three::PerspectiveCamera,
    renderer: &'static three::WebGLRenderer,
    materials: Vec<&'static three::MeshBasicMaterial>,
    canvas: &'static HTMLCanvasElement,
}

impl Drawing {
    fn draw(&self) {
        let width = self.canvas.client_width().max(1.0);
        let height = self.canvas.client_height().max(1.0);
        self.renderer
            .set_pixel_ratio(window.device_pixel_ratio().min(2.0));
        self.renderer.set_size(width, height, false);
        self.camera.set_aspect(width / height);
        self.camera.update_projection_matrix();
        self.renderer.render(self.scene, self.camera);
    }

    fn show(&self, facelets: &str, view: &str) {
        for (material, value) in self.materials.iter().zip(facelets.chars()) {
            material.color().set_hex(color(value).0);
        }
        if view == "front" {
            self.camera.position().set(0.0, 0.0, 9.5);
        } else {
            self.camera.position().set(6.2, 5.0, 7.0);
        }
        self.camera.look_at(0.0, 0.0, 0.0);
        self.draw();
    }
}

pub fn Preview(PreviewProps { view, facelets }: PreviewProps) -> JSX::Element {
    let canvas = use_ref(None::<&'static HTMLCanvasElement>);
    let drawing = use_ref(None::<Rc<Drawing>>);
    let (problem, set_problem) = use_state(|| None::<&'static str>);

    // Renderer lifetime belongs to the canvas, not camera or cube updates.
    use_effect(
        move || -> Option<Box<dyn FnOnce()>> {
            let canvas = canvas.current().expect("mounted canvas");
            match three::renderer(RendererOptions {
                canvas,
                antialias: true,
                alpha: true,
            }) {
                Ok(renderer) => {
                    let scene = three::scene();
                    let camera = three::perspective_camera(35.0, 1.0, 0.1, 100.0);
                    let body_geometry = three::box_geometry(3.0, 3.0, 3.0);
                    let black = three::material(MaterialOptions { color: 0x151a17 });
                    scene.add(three::mesh(body_geometry, black));
                    let geometry = three::plane_geometry(0.92, 0.92);
                    let mut materials = Vec::with_capacity(54);
                    for (face, normal) in NORMALS.iter().enumerate() {
                        for cell in 0..9 {
                            let material = three::material(MaterialOptions {
                                color: color(FACES[face]).0,
                            });
                            let sticker = three::mesh(geometry, material);
                            let position = facelet_position(face, cell);
                            sticker.position().set(
                                f64::from(position[0]) + f64::from(normal[0]) * 0.505,
                                f64::from(position[1]) + f64::from(normal[1]) * 0.505,
                                f64::from(position[2]) + f64::from(normal[2]) * 0.505,
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
                            scene.add(sticker);
                            materials.push(material);
                        }
                    }
                    let objects = Rc::new(Drawing {
                        scene,
                        camera,
                        renderer,
                        materials,
                        canvas,
                    });
                    drawing.set_current(Some(Rc::clone(&objects)));
                    let resized = Rc::clone(&objects);
                    let resize = webapi::listener(move |_: &Event| resized.draw());
                    let lost = webapi::listener(move |event: &Event| {
                        event.prevent_default();
                        set_problem.set(Some("Compatible view · graphics context lost"));
                    });
                    window.add_event_listener(events::Resize, resize, None);
                    canvas.add_event_listener_named("webglcontextlost", lost, None);
                    Some(Box::new(move || {
                        window.remove_event_listener(events::Resize, resize, None);
                        canvas.remove_event_listener_named("webglcontextlost", lost, None);
                        drawing.set_current(None);
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
                    None
                }
            }
        },
        Some(&()),
    );

    let shown = facelets.clone();
    use_effect(
        move || {
            if let Some(drawing) = drawing.current() {
                drawing.show(&shown, view);
            }
        },
        Some(&(view, facelets.clone())),
    );

    let values: Vec<char> = facelets.chars().collect();
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
