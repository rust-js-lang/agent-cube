//! The Three.js surface used by the preview, declared against three@0.187.0.
use std::marker::PhantomData;

use js::{JsError, JsObject};
use webapi::HTMLCanvasElement;

#[cfg_attr(rust_js, rust_js::types = "three#Scene")]
pub struct Scene(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#Group")]
pub struct Group(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#PerspectiveCamera")]
pub struct PerspectiveCamera(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#WebGLRenderer")]
pub struct WebGLRenderer(PhantomData<JsObject>);
#[cfg_attr(
    rust_js,
    rust_js::types = "three/addons/geometries/RoundedBoxGeometry.js#RoundedBoxGeometry"
)]
pub struct RoundedBoxGeometry(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#MeshBasicMaterial")]
pub struct MeshBasicMaterial(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#MeshPhongMaterial")]
pub struct MeshPhongMaterial(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#Mesh")]
pub struct Mesh(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#AmbientLight")]
pub struct AmbientLight(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#DirectionalLight")]
pub struct DirectionalLight(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#Color")]
pub struct Color(PhantomData<JsObject>);
#[cfg_attr(rust_js, rust_js::types = "three#Euler")]
pub struct Euler(PhantomData<JsObject>);
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
pub struct PhongOptions {
    pub color: u32,
    pub shininess: f64,
    pub specular: u32,
}

#[cfg_attr(rust_js, rust_js::link_name = "new three#Scene")]
pub fn scene() -> &'static Scene {
    unreachable!()
}
#[cfg_attr(rust_js, rust_js::link_name = "new three#Group")]
pub fn group() -> &'static Group {
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
#[cfg_attr(
    rust_js,
    rust_js::link_name = "new three/addons/geometries/RoundedBoxGeometry.js#RoundedBoxGeometry"
)]
pub fn rounded_box(
    width: f64,
    height: f64,
    depth: f64,
    segments: u32,
    radius: f64,
) -> &'static RoundedBoxGeometry {
    unreachable!()
}
#[cfg_attr(rust_js, rust_js::link_name = "new three#MeshBasicMaterial")]
pub fn material(options: MaterialOptions) -> &'static MeshBasicMaterial {
    unreachable!()
}
#[cfg_attr(rust_js, rust_js::link_name = "new three#MeshPhongMaterial")]
pub fn phong_material(options: PhongOptions) -> &'static MeshPhongMaterial {
    unreachable!()
}
#[cfg_attr(rust_js, rust_js::link_name = "new three#Mesh")]
pub fn mesh<G: Geometry, M: Material>(geometry: &G, material: &M) -> &'static Mesh {
    unreachable!()
}
#[cfg_attr(rust_js, rust_js::link_name = "new three#AmbientLight")]
pub fn ambient_light(color: u32, intensity: f64) -> &'static AmbientLight {
    unreachable!()
}
#[cfg_attr(rust_js, rust_js::link_name = "new three#DirectionalLight")]
pub fn directional_light(color: u32, intensity: f64) -> &'static DirectionalLight {
    unreachable!()
}

/// The shared Object3D surface of these actual Three.js subclasses.
pub trait Node {
    #[cfg_attr(rust_js, rust_js::link_name = "add")]
    fn add<T: Node>(&self, node: &T) -> &Self {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get position")]
    fn position(&self) -> &'static Vector3 {
        unreachable!()
    }
    #[cfg_attr(rust_js, rust_js::link_name = "get rotation")]
    fn rotation(&self) -> &'static Euler {
        unreachable!()
    }
}
impl Node for Scene {}
impl Node for Group {}
impl Node for Mesh {}
impl Node for PerspectiveCamera {}
impl Node for AmbientLight {}
impl Node for DirectionalLight {}

impl PerspectiveCamera {
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
    pub fn set(&self, x: f64, y: f64, z: f64) -> &Self {
        unreachable!()
    }
}
impl Euler {
    #[cfg_attr(rust_js, rust_js::link_name = "set")]
    pub fn set(&self, x: f64, y: f64, z: f64) -> &Self {
        unreachable!()
    }
}
impl WebGLRenderer {
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
impl MeshBasicMaterial {
    #[cfg_attr(rust_js, rust_js::link_name = "get color")]
    pub fn color(&self) -> &'static Color {
        unreachable!()
    }
}
impl Color {
    #[cfg_attr(rust_js, rust_js::link_name = "setHex")]
    pub fn set_hex(&self, hex: u32) -> &Self {
        unreachable!()
    }
}
pub trait Material {
    #[cfg_attr(rust_js, rust_js::link_name = "dispose")]
    fn dispose(&self) {
        unreachable!()
    }
}
impl Material for MeshBasicMaterial {}
impl Material for MeshPhongMaterial {}
pub trait Geometry {
    #[cfg_attr(rust_js, rust_js::link_name = "dispose")]
    fn dispose(&self) {
        unreachable!()
    }
}
impl Geometry for RoundedBoxGeometry {}
