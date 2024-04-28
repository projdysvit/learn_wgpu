use bytemuck::{Pod, Zeroable};
use cgmath::{perspective, Deg, Matrix4, Point3, SquareMatrix, Vector3};

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
