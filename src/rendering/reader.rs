
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

// Glob and find all shaders with the given name. Go through and build shader
// program with version 410 and 320 (Newest GLSL versions on OS X). If one of
// them does not exist give a warning. If none exist panic.
fn load(shader_name: &str) -> Program {
    let path = Path::new("assets/");
    let disp = path.display();

    let mut file = match File::open(&path) {
        Err(reason) => panic!("Could not open file {}: {}",
                              disp,
                              reason.description()),
        Ok(file) => file,
    };

    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Err(reason) => panic!("Could not read file {}: {}", disp, reason.description()),
        Ok(_) => {},
    };


}