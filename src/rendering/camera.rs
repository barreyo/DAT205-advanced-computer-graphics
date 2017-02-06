
#[macro_use()]
extern crate cgmath;

use cgmath::{vector, matrix, projection};

#[derive(Debug)]
struct Perspective {
    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32
}

pub struct Camera {
    position: Vector3<f32>,
    
}

