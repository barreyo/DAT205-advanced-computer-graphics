
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
    // Clear color
    SetClearColor(f32, f32, f32),
    // FXAA AA
    ToggleFXAA,

    // * --- WindowEvent
    // Resize the window
    SetWindowSize(f32, f32),
    // Toggle fullscreen
    ToggleFullscreen,
    // VSync
    ToggleVSync,

    // * --- EntityEvent
    // Instantly move camera to position
    SetCameraPos(f32, f32),
    MoveCamera(bool, bool, bool, bool),

    // * --- UIEvent
    // Show message in console
    ConsoleMessage(String, ConsoleLogLevel),
    // Toggle display of console
    ToggleConsole,
}
