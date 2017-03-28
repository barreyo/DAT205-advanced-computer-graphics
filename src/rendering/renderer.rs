
use rendering::camera::Camera;
use rendering::colors::Color;

#[derive(Debug, Clone)]
pub enum RenderMode {
    WIREFRAME,
    DEFAULT,
}

struct Renderer {
    camera:         &'static Camera,
    clear_color:    Color,
    // passes:    Vec<RenderPass>,
    mode:           RenderMode,
}


