use bytemuck::{Pod, Zeroable};
use cgmath::{perspective, Deg, Matrix4, Point3, Vector3};
use winit::{event::{ElementState, KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32>
    {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4]
}

impl CameraUniform {
    pub fn new() -> Self
    {
        use cgmath::SquareMatrix;

        Self {
            view_proj: Matrix4::identity().into()
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera)
    {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state,
                    physical_key: PhysicalKey::Code(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}

