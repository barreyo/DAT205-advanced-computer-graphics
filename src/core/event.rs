
use ui::console::ConsoleLogLevel;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EventID {
    RenderEvent,
    WindowEvent,
    EntityEvent,
    UIEvent,
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
    SetWindowSize(u32, u32), // USE LIST FOR ARGUMENTS? ONLY INTS?
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
