//! The Three.js surface used by the preview, declared against three@0.187.0.
use std::marker::PhantomData;

use js::{JsError, JsObject};
use webapi::HTMLCanvasElement;

#[cfg_attr(rust_js, rust_js::types = "three#Scene")]
pub struct Scene(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#PerspectiveCamera")]
pub struct PerspectiveCamera(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#WebGLRenderer")]
pub struct WebGLRenderer(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#BoxGeometry")]
pub struct BoxGeometry(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#MeshBasicMaterial")]
pub struct MeshBasicMaterial(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#Mesh")]
pub struct Mesh(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#Vector3")]
pub struct Vector3(PhantomData<JsObject>);

pub struct RendererOptions {
    pub canvas: &'static HTMLCanvasElement,
    pub antialias: bool,
    pub alpha: bool,
}

pub struct MaterialOptions {
    pub color: u32,
}

#[cfg_attr(rust_js, rust_js::link_name = "new three#Scene")]
pub fn scene() -> &'static Scene {
    unreachable!()
}
#[cfg_attr(rust_js, rust_js::link_name = "new three#PerspectiveCamera")]
pub fn perspective_camera(
    fov: f64,
    aspect: f64,
    near: f64,
    far: f64,
) -> &'static PerspectiveCamera {
    unreachable!()
}
#[cfg_attr(rust_js, rust_js::link_name = "new three#WebGLRenderer")]
pub fn renderer(options: RendererOptions) -> Result<&'static WebGLRenderer, &'static JsError> {
    unreachable!()
}
#[cfg_attr(rust_js, rust_js::link_name = "new three#BoxGeometry")]
pub fn box_geometry(width: f64, height: f64, depth: f64) -> &'static BoxGeometry {
    unreachable!()
}
#[cfg_attr(rust_js, rust_js::link_name = "new three#MeshBasicMaterial")]
pub fn material(options: MaterialOptions) -> &'static MeshBasicMaterial {
    unreachable!()
}
#[cfg_attr(rust_js, rust_js::link_name = "new three#Mesh")]
pub fn mesh(geometry: &BoxGeometry, materials: Vec<&MeshBasicMaterial>) -> &'static Mesh {
    unreachable!()
}

impl Scene {
    #[cfg_attr(rust_js, rust_js::link_name = "add")]
    pub fn add(&self, mesh: &Mesh) {
        unreachable!()
    }
}
impl Mesh {
    #[cfg_attr(rust_js, rust_js::link_name = "get position")]
    pub fn position(&self) -> &'static Vector3 {
        unreachable!()
    }
}
impl PerspectiveCamera {
    #[cfg_attr(rust_js, rust_js::link_name = "get position")]
    pub fn position(&self) -> &'static Vector3 {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "lookAt")]
    pub fn look_at(&self, x: f64, y: f64, z: f64) {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "set aspect")]
    pub fn set_aspect(&self, aspect: f64) {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "updateProjectionMatrix")]
    pub fn update_projection_matrix(&self) {
        unreachable!()
    }
}
impl Vector3 {
    #[cfg_attr(rust_js, rust_js::link_name = "set")]
    pub fn set(&self, x: f64, y: f64, z: f64) -> &'static Vector3 {
        unreachable!()
    }
}
impl WebGLRenderer {
    #[cfg_attr(rust_js, rust_js::link_name = "forceContextLoss")]
    pub fn force_context_loss(&self) {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "setSize")]
    pub fn set_size(&self, width: f64, height: f64, update_style: bool) {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "setPixelRatio")]
    pub fn set_pixel_ratio(&self, ratio: f64) {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "render")]
    pub fn render(&self, scene: &Scene, camera: &PerspectiveCamera) {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "dispose")]
    pub fn dispose(&self) {
        unreachable!()
    }
}
impl BoxGeometry {
    #[cfg_attr(rust_js, rust_js::link_name = "dispose")]
    pub fn dispose(&self) {
        unreachable!()
    }
}
impl MeshBasicMaterial {
    #[cfg_attr(rust_js, rust_js::link_name = "dispose")]
    pub fn dispose(&self) {
        unreachable!()
    }
}
