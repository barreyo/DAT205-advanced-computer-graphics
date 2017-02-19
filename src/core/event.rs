
use ui::console::ConsoleLogLevel;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EventID {
    RenderEvent = 0,
    WindowEvent = 1,
    EntityEvent = 2,
    UIEvent     = 3,
}

#[derive(Clone, Debug)]
pub enum Event {
    // * --- RenderEvent
    // Reload shaders
    ReloadShaders,
    // Toggle wireframe mode
    ToggleWireframe,

    // * --- WindowEvent
    // Resize the window
    SetWindowSize(u32, u32),
    // Toggle fullscreen
    ToggleFullscreen,
    // VSync
    ToggleVSync,

    // * --- EntityEvent
    // Instantly move camera to position
    MoveCamera(f32, f32),

    // * --- UIEvent
    // Show message in console
    ConsoleMessage(String, ConsoleLogLevel),

    // Toggle display of console
    ToggleConsole,
}
