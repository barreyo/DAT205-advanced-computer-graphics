
use alewife;
use core::event;

use na::{Point3, Vector2, Vector3, Matrix4, Isometry3, Perspective3, Translation3};
use na;

use conrod::backend::glium::glium;

pub struct Camera {
    event_queue:        alewife::Subscriber<event::EventID, event::Event>,
    eye:                Point3<f32>,
    pitch:              f32,
    yaw:                f32,
    speed:              f32,
    rotate_speed:       f32,
    projection:         Perspective3<f32>,
    inv_proj_view:      Matrix4<f32>,
    proj_view:          Matrix4<f32>,
    prev_mouse_pos:     Vector2<f32>,
}

// TODO: Create a camera builder so perspective settings etc can be tweaked.
impl Camera {

    pub fn new(fov : f32, ratio : f32, pos: Point3<f32>,
               e_que: alewife::Subscriber<event::EventID, event::Event>) -> Camera {
        Camera {
            event_queue:    e_que,
            eye:            Point3::new(0.0, 0.0, 0.0),
            pitch:          0.0,
            yaw:            0.0,
            speed:          0.1,
            rotate_speed:   0.005,
            projection:     Perspective3::new(ratio, fov, 0.01, 10000.0),
            inv_proj_view:  na::zero(),
            proj_view:      na::zero(),
            prev_mouse_pos: na::zero(),
        }
    }

    pub fn get_view_proj(&self) -> Matrix4<f32> {
        self.proj_view
    }

    pub fn get_inv_view_proj(&self) -> Matrix4<f32> {
        self.inv_proj_view
    }

    pub fn get_eye(&self) -> Point3<f32> {
        self.eye
    }

    pub fn set_eye(&mut self, eye: Point3<f32>) {
        self.eye = eye;
        self.update_proj_view();
    }

    pub fn set_pitch_deg(&mut self, angle: f32) {

    }

    pub fn set_yaw_deg(&mut self, angle: f32) {

    }

    pub fn set_pitch_rad(&mut self, angle: f32) {

    }

    pub fn set_yaw_rad(&mut self, angle: f32) {

    }

    pub fn at(&self) -> Point3<f32> {
        let ax = self.eye.x + self.yaw.cos() * self.pitch.sin();
        let ay = self.eye.y + self.pitch.cos();
        let az = self.eye.z + self.yaw.sin() * self.pitch.sin();

        Point3::new(ax, ay, az)
    }

    fn view_transform(&self) -> Isometry3<f32> {
        Isometry3::look_at_rh(&self.eye, &self.at(), &Vector3::y())
    }

    pub fn look_at(&mut self, eye: Point3<f32>, pos: Point3<f32>) {
        // Squared euclidian norm is faster to calculate
        let d = na::distance(&eye, &pos);

        let n_pitch  = ((pos.y - eye.y) / d).acos();
        let n_yaw    = (pos.z - eye.z).atan2(pos.x - eye.x);

        self.eye     = eye;
        self.yaw     = n_yaw;
        self.pitch   = n_pitch;
        self.update_proj_view();
    }

    pub fn translate(&mut self, t: &Translation3<f32>) {
        let n_eye = t * self.eye;
        self.set_eye(n_eye);
    }

    fn update_proj_view(&mut self) {
        self.proj_view = *self.projection.as_matrix() *
                          self.view_transform().to_homogeneous();
        // If determinant is 0, aka we cant take inverse, we get None.
        // TODO: work around this instead of ignoring failed inversion.
        if let Some(inv) = self.proj_view.try_inverse() {
            self.inv_proj_view = inv;
        }
    }

    fn handle_rotate(&mut self, delta: Vector2<f32>) {
        self.yaw    = self.yaw   + delta.x * self.rotate_speed;
        self.pitch  = self.pitch + delta.y * self.rotate_speed;
        self.update_proj_view();
    }

    fn handle_input(&mut self, left: bool, right: bool, up: bool, down: bool) -> Vector3<f32> {

        let transf   = self.view_transform();
        let vforward = transf * Vector3::z();
        let vright   = transf * Vector3::x();

        let mut mvm = na::zero::<Vector3<f32>>();

        if left {
            mvm = mvm + vright
        }
        if right {
            mvm = mvm - vright
        }
        if up {
            mvm = mvm + vforward
        }
        if down {
            mvm = mvm - vforward
        }

        if let Some(normalized) = mvm.try_normalize(1.0e-10) {
            normalized
        } else {
            mvm
        }
    }

    pub fn update(&mut self, window_evts: &Vec<glium::glutin::Event>) {

        let events: Vec<_>      = self.event_queue.fetch();
        let mut left            = false;
        let mut right           = false;
        let mut up              = false;
        let mut down            = false;
        let mut rotate_button   = false;

        for event in events {
            match event {
                (_, event::Event::SetCameraPos(x, y)) => info!("Move camera lol"),
                _ => {},
            }
        }

        let mut cur_mouse_pos = self.prev_mouse_pos;
        for evt in window_evts {
            match *evt {
                glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Pressed, _, Some(glium::glutin::VirtualKeyCode::W)) => {
                    up = true;
                },
                glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Pressed, _, Some(glium::glutin::VirtualKeyCode::A)) => {
                    left = true;
                },
                glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Pressed, _, Some(glium::glutin::VirtualKeyCode::S)) => {
                    down = true;
                },
                glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Pressed, _, Some(glium::glutin::VirtualKeyCode::D)) => {
                    right = true;
                },
                glium::glutin::Event::MouseInput(glium::glutin::ElementState::Pressed, glium::glutin::MouseButton::Left) => {
                    rotate_button = true;
                },
                glium::glutin::Event::MouseMoved(x, y) => {
                    cur_mouse_pos = Vector2::new(x as f32, y as f32);
                },
                _ => {},
            }
        }

        if rotate_button {
            let mouse_delta = cur_mouse_pos - self.prev_mouse_pos;
            self.handle_rotate(mouse_delta);
            self.prev_mouse_pos = cur_mouse_pos;
        }

        let mvm_dir   = self.handle_input(left, right, up, down);
        let mvm       = mvm_dir * self.speed;

        self.translate(&Translation3::from_vector(mvm));
    }
}
