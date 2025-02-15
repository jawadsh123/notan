use crate::batch::*;
use crate::manager::process_pipeline;
use notan_graphics::prelude::*;
use notan_macro::{fragment_shader, vertex_shader};

//language=glsl
const SHAPES_VERTEX: ShaderSource = vertex_shader! {
    r#"
    #version 450
    layout(location = 0) in vec2 a_pos;
    layout(location = 1) in vec4 a_color;

    layout(location = 0) out vec4 v_color;
    layout(set = 0, binding = 0) uniform Locals {
        mat4 u_projection;
    };

    void main() {
        v_color = a_color;
        gl_Position = u_projection * vec4(a_pos, 0.0, 1.0);
    }
    "#
};

//language=glsl
const SHAPES_FRAGMENT: ShaderSource = fragment_shader! {
    r#"
    #version 450
    precision mediump float;

    layout(location = 0) in vec4 v_color;
    layout(location = 0) out vec4 color;

    void main() {
        color = v_color;
    }
    "#
};

fn vertex_info() -> VertexInfo {
    VertexInfo::new()
        .attr(0, VertexFormat::Float32x2)
        .attr(1, VertexFormat::Float32x4)
}

pub fn create_shape_pipeline(
    device: &mut Device,
    fragment: Option<&ShaderSource>,
) -> Result<Pipeline, String> {
    let fragment = fragment.unwrap_or(&SHAPES_FRAGMENT);
    device
        .create_pipeline()
        .from(&SHAPES_VERTEX, fragment)
        .with_vertex_info(&vertex_info())
        .with_color_blend(BlendMode::NORMAL)
        .build()
}

pub(crate) struct ShapePainter {
    vbo: Buffer,
    ebo: Buffer,
    ubo: Buffer,
    pipeline: Pipeline,
    vertices: Vec<f32>,
    indices: Vec<u32>,
    uniforms: [f32; 16],
    count_vertices: usize,
    count_indices: usize,
    dirty_buffer: bool,
}

impl ShapePainter {
    pub fn new(device: &mut Device) -> Result<Self, String> {
        let pipeline = create_shape_pipeline(device, None)?;

        let uniforms = [0.0; 16];
        let vbo = device
            .create_vertex_buffer()
            .with_info(&vertex_info())
            .build()?;

        let ebo = device.create_index_buffer().build()?;

        let ubo = device
            .create_uniform_buffer(0, "Locals")
            .with_data(&uniforms)
            .build()?;

        Ok(Self {
            vbo,
            ebo,
            ubo,
            pipeline,
            vertices: vec![],
            indices: vec![],
            uniforms,
            count_indices: 0,
            count_vertices: 0,
            dirty_buffer: false,
        })
    }

    pub fn push(&mut self, renderer: &mut Renderer, batch: &Batch, projection: &notan_math::Mat4) {
        if let BatchType::Shape = &batch.typ {
            process_pipeline(renderer, batch, &self.pipeline);

            let len = (self.count_vertices / self.pipeline.offset()) as u32;
            let offset = self.count_indices;

            self.indices.extend(batch.indices.iter().map(|i| i + len));
            self.count_indices = self.indices.len();

            self.vertices.extend(&batch.vertices);
            self.count_vertices = self.vertices.len();

            self.uniforms.copy_from_slice(&projection.to_cols_array());

            renderer.bind_buffers(&[&self.vbo, &self.ebo, &self.ubo]);
            renderer.draw(offset as _, batch.indices.len() as _);
            self.dirty_buffer = true;
        }
    }

    #[inline]
    pub fn upload_buffers(&mut self, device: &mut Device) {
        if self.dirty_buffer {
            self.dirty_buffer = false;
            device.set_buffer_data(&self.vbo, &self.vertices);
            device.set_buffer_data(&self.ebo, &self.indices);
            device.set_buffer_data(&self.ubo, &self.uniforms);
        }
    }

    pub fn clear(&mut self) {
        self.count_vertices = 0;
        self.count_indices = 0;
        self.vertices.clear();
        self.indices.clear();
    }
}
