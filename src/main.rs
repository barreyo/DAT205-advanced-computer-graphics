#[macro_use]
extern crate conrod;
extern crate glutin;
#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_window_glutin;

extern crate genmesh;
extern crate noise;
extern crate rand;

extern crate obj;

#[macro_use]
extern crate lazy_static;
extern crate find_folder;

#[macro_use]
extern crate log;

extern crate approx; // relative_eq!
extern crate nalgebra as na;

extern crate alewife;

mod core;
// mod input;
mod rendering;
mod support;
mod ui;

fn main() {
    use core::core::core::init;
    init();
}
