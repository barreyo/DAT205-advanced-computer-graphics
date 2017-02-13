
#[macro_use()]
extern crate cgmath;

use cgmath::{vector, matrix, projection};

#[derive(Debug)]
pub struct PerspectiveParams {
    fov:            f32,
    aspect_ratio:   f32,
    near:           f32,
    far:            f32
}

pub struct Camera {
    position:   Vector3<f32>,
    up:         Vector3<f32>,
    right:      Vector3<f32>,
    forward:    Vector3<f32>,
    pitch:      f32,
    yaw:        f32,
    speed:      f32,
    pers:       Perspective
}

impl Camera {

    pub fn new(fov : f32, ratio : f32) -> Camera {
        Camera {
            position: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            right: [0.0, 0.0, 1.0],
            forward: [-1.0, -1.0, -1.0],
            pitch:
        }
    }

    pub fn update(&mut self) {

    }
}
