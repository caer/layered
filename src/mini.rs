/// ## Unstable
///
/// 2D rendering experiments using lower-level OpenGL/WebGL
/// APIs provided by Miniquad in lieu of higher-level APIs
/// provided by Macroquad.
use miniquad::*;

/// Texture used for testing image rendering.
const TESTURE: &[u8] = include_bytes!("../assets/splash.png");

const MIN_CLIP_X: f32 = MAX_CLIP_X * -1.0;
const MAX_CLIP_X: f32 = 1.0;
const MIN_CLIP_Y: f32 = MAX_CLIP_Y * -1.0;
const MAX_CLIP_Y: f32 = 1.0;

/// Entrypoint which spawns a default
/// miniquad instance with a new [`Stage`].
fn main() {
    miniquad::start(Default::default(), move || Box::new(Stage::new()));
}

/// Rendering pipeline.
struct Stage {
    ctx: Box<dyn RenderingBackend>,
    pipeline: Pipeline,
    bindings: Bindings,
    testure_aspect_ratio: f32,
}

impl Stage {
    pub fn new() -> Stage {
        // Provision a new rendering backend.
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        // Load the texture into bytes in-memory.
        let texture = image::load_from_memory(TESTURE).unwrap();
        let texture_rgba8 = texture.flipv().to_rgba8();
        let width = texture_rgba8.width() as u16;
        let height = texture_rgba8.height() as u16;
        let bytes = texture_rgba8.into_raw();
        let testure_aspect_ratio = width as f32 / height as f32;

        // Load the texture bytes into the rendering backend.
        let texture = ctx.new_texture_from_rgba8(width, height, &bytes);

        // Declare a vertex buffer for rendering a texture
        // onto a rectangle occupying clip space.
        #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex { pos : (MIN_CLIP_X, MIN_CLIP_Y), uv: (0.0, 0.0) },
            Vertex { pos : (MAX_CLIP_X, MIN_CLIP_Y), uv: (1.0, 0.0) },
            Vertex { pos : (MAX_CLIP_X, MAX_CLIP_X), uv: (1.0, 1.0) },
            Vertex { pos : (MIN_CLIP_X, MAX_CLIP_Y), uv: (0.0, 1.0) },
        ];
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        // Declare the point indices of the vertices.
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        // Bind vertices and textures to the context.
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![texture],
        };

        // Compile shaders.
        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: VERTEX_SHADER,
                        fragment: FRAGMENT_SHADER,
                    },
                    backend => unimplemented!("unsupported backend: {backend:?}"),
                },
                shader_meta(),
            )
            .unwrap();

        // Create a rendering pipeline with the compiled shaders.
        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
            ],
            shader,
            PipelineParams::default(),
        );

        Stage {
            pipeline,
            bindings,
            ctx,
            testure_aspect_ratio,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        let screen_size = window::screen_size();

        // TODO: ?
        self.ctx.begin_default_pass(Default::default());

        // Clear the screen.
        self.ctx.clear(
            Some((
                layered::color::BACKGROUND.red as f32 / 255.,
                layered::color::BACKGROUND.green as f32 / 255.,
                layered::color::BACKGROUND.blue as f32 / 255.,
                layered::color::BACKGROUND.alpha as f32 / 255.,
            )),
            None,
            None,
        );

        // TODO: ?
        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);

        // Calculate uniforms to scale image to fit in
        // screenspace while preserving the aspect ratio.
        let canvas_aspect = screen_size.0 / screen_size.1;
        let mut scale_y = 1.0;
        let mut scale_x = self.testure_aspect_ratio / canvas_aspect;
        if scale_x > 1.0 {
            scale_y = 1.0 / scale_x;
            scale_x = 1.0;
        }

        // Draw image.
        self.ctx
            .apply_uniforms(UniformsSource::table(&ShaderUniforms {
                offset: (0.0, 0.0),
                scale: (scale_x, scale_y),
            }));
        self.ctx.draw(0, 6, 1);

        // TODO: ?
        self.ctx.end_render_pass();

        // TODO: ?
        self.ctx.commit_frame();
    }
}

/// ? TODO:
///
/// - An `attribute` is auto-injected by miniquad
///   for each attribute declared in [`RenderingBackend::new_pipeline`].
///   Attributes must be declared in the order and type
///   they appear in the corresponding [`Vertex`] type.
pub const VERTEX_SHADER: &str = r#"#version 100
attribute vec2 in_pos;
attribute vec2 in_uv;

uniform vec2 offset;
uniform vec2 scale;

varying lowp vec2 texcoord;

void main() {
    vec2 position = scale * (offset + in_pos);

    gl_Position = vec4(position, 0, 1);
    
    texcoord = in_uv;
}"#;

/// ? TODO:
///
/// - A `uniform sampler2D` is auto-injected by miniquad
///   for each image in [`ShaderMeta::images`].
pub const FRAGMENT_SHADER: &str = r#"#version 100
varying lowp vec2 texcoord;

uniform sampler2D tex;

void main() {
    gl_FragColor = texture2D(tex, texcoord);
}"#;

/// A single vertex in a [`BufferType::VertexBuffer`].
#[repr(C)]
struct Vertex {
    /// Vertex coordinate (in clip space) as a
    /// `-1` to `1` normalized `x, y` tuple.
    pos: (f32, f32),

    /// Vertex coordinate (in local texture space) as a
    /// `0` to `1` normalized `x, y` tuple.
    uv: (f32, f32),
}

/// "Uniforms" (variables) we can modify between draw calls to the GPU.
#[repr(C)]
pub struct ShaderUniforms {
    /// Drawing offset of the texture.
    pub offset: (f32, f32),

    /// Drawing scale of the texture.
    pub scale: (f32, f32),
}

/// Returns a [`ShaderMeta`] for [`ShaderUniforms`].
pub fn shader_meta() -> ShaderMeta {
    ShaderMeta {
        images: vec!["tex".to_string()],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("offset", UniformType::Float2),
                UniformDesc::new("scale", UniformType::Float2),
            ],
        },
    }
}
