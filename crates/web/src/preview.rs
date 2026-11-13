use react::{JSX, jsx, use_effect, use_ref, use_state};
use webapi::{Event, EventTargetExt, HTMLCanvasElement, events, window};

use crate::three::{self, MaterialOptions, RendererOptions};

pub struct PreviewProps {
    pub view: &'static str,
}

pub fn Preview(PreviewProps { view }: PreviewProps) -> JSX::Element {
    let canvas = use_ref(None::<&'static HTMLCanvasElement>);
    let (problem, set_problem) = use_state(|| None::<String>);

    use_effect(
        move || -> Option<Box<dyn FnOnce()>> {
            let canvas = canvas.current().expect("mounted canvas");
            let scene = three::scene();
            let camera = three::perspective_camera(35.0, 1.0, 0.1, 100.0);
            let renderer = three::renderer(RendererOptions {
                canvas,
                antialias: true,
                alpha: true,
            });
            match renderer {
                Ok(renderer) => {
                    let geometry = three::box_geometry(0.92, 0.92, 0.92);
                    // BoxGeometry's material order is +X, -X, +Y, -Y, +Z, -Z.
                    let materials: Vec<_> =
                        [0xe45e40, 0xf1a84b, 0xf1efdf, 0xe9d35f, 0x95c975, 0x649cbf]
                            .into_iter()
                            .map(|color| three::material(MaterialOptions { color }))
                            .collect();
                    for x in -1..=1 {
                        for y in -1..=1 {
                            for z in -1..=1 {
                                if x != 0 || y != 0 || z != 0 {
                                    let piece = three::mesh(geometry, materials.clone());
                                    piece.position().set(x.into(), y.into(), z.into());
                                    scene.add(piece);
                                }
                            }
                        }
                    }
                    if view == "front" {
                        camera.position().set(0.0, 0.0, 9.5);
                    } else {
                        camera.position().set(6.2, 5.0, 7.0);
                    }
                    camera.look_at(0.0, 0.0, 0.0);
                    let draw = move || {
                        let width = canvas.client_width().max(1.0);
                        let height = canvas.client_height().max(1.0);
                        renderer.set_pixel_ratio(window.device_pixel_ratio().min(2.0));
                        renderer.set_size(width, height, false);
                        camera.set_aspect(width / height);
                        camera.update_projection_matrix();
                        renderer.render(scene, camera);
                    };
                    draw();
                    let resize = webapi::listener(move |_: &Event| draw());
                    window.add_event_listener(events::Resize, resize, None);
                    set_problem.set(None);
                    Some(Box::new(move || {
                        window.remove_event_listener(events::Resize, resize, None);
                        geometry.dispose();
                        for material in materials {
                            material.dispose();
                        }
                        renderer.dispose();
                        renderer.force_context_loss();
                    }))
                }
                Err(_) => {
                    set_problem.set(Some("The 3D preview needs WebGL. Camera controls and agent tools remain available.".to_string()));
                    None
                }
            }
        },
        Some(&(view,)),
    );

    jsx! {
        <div className="cube-preview" data-view={view}>
            <canvas ref={canvas} aria-label={format!("Solved cube preview, {view} view")} role="img" />
            {problem.as_ref().map(|message| jsx! { <p role="alert">{message}</p> })}
        </div>
    }
}
