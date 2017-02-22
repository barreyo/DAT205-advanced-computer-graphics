

pub enum RenderMode {
    WIREFRAME,
    DEFAULT,
}

#[derive(Debug)]
struct Renderer {
    camera: &'static Camera,
    clear_color: Color,
    passes: Vec<RenderPass>,
    mode: RenderMode,
}

struct RendererBuilder {
    camera: &Camera,
    clear_color: Color,
    mode: RenderMode,

}