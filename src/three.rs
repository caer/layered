//! ## Unstable
//!
//! 3D graphics experiments, for perhaps one day
//! having a tool to convert 3D models to isometric
//! 2D spritesheets (a la [https://pixelover.io/]).
//!
//! The code in this module at the time
//! of this commit borrows heavily from the work
//! (here)[https://github.com/whoisryosuke/wgpu-hello-world/blob/play/gltf-r2/src/shader.wgsl].
use std::{
    io::{BufReader, Cursor},
    path::Path,
    sync::{atomic::AtomicBool, Arc},
};

use codas_flow::async_support::yield_now;
use gltf::Gltf;
use model::{AnimationClip, Keyframes, Mesh, Model, ModelVertex};
use wgpu::{util::DeviceExt, Maintain};

mod model;
mod texture;

const OUTPUT_PATH: &str = "render.png";
const TEXTURE_DIMS: (usize, usize) = (512, 512);

pub async fn render_to_image() {
    // This will later store the raw pixel value data locally. We'll create it now as
    // a convenient size reference.
    let mut texture_data = Vec::<u8>::with_capacity(TEXTURE_DIMS.0 * TEXTURE_DIMS.1 * 4);

    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        )
        .await
        .unwrap();

    let shader = device.create_shader_module(wgpu::include_wgsl!("three/shader.wgsl"));

    let render_target = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: TEXTURE_DIMS.0 as u32,
            height: TEXTURE_DIMS.1 as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
    });
    let output_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: texture_data.capacity() as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::TextureFormat::Rgba8UnormSrgb.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    eprintln!("Wgpu context set up.");

    //-----------------------------------------------

    let texture_view = render_target.create_view(&wgpu::TextureViewDescriptor::default());

    let mut command_encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        render_pass.set_pipeline(&pipeline);
        render_pass.draw(0..3, 0..1);
    }
    // The texture now contains our rendered image
    command_encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &render_target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_staging_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                // This needs to be a multiple of 256. Normally we would need to pad
                // it but we here know it will work out anyways.
                bytes_per_row: Some((TEXTURE_DIMS.0 * 4) as u32),
                rows_per_image: Some(TEXTURE_DIMS.1 as u32),
            },
        },
        wgpu::Extent3d {
            width: TEXTURE_DIMS.0 as u32,
            height: TEXTURE_DIMS.1 as u32,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(command_encoder.finish()));
    eprintln!("Commands submitted.");

    //-----------------------------------------------

    // Time to get our image.
    let buffer_slice = output_staging_buffer.slice(..);
    let signal = Arc::new(AtomicBool::new(false));
    let signal_clone = signal.clone();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
        signal_clone.store(true, std::sync::atomic::Ordering::Relaxed)
    });
    device.poll(Maintain::Wait).panic_on_timeout();
    while !signal.load(std::sync::atomic::Ordering::Relaxed) {
        yield_now().await;
    }
    eprintln!("Output buffer mapped.");
    {
        let view = buffer_slice.get_mapped_range();
        texture_data.extend_from_slice(&view[..]);
    }
    eprintln!("Image data copied to local.");
    output_staging_buffer.unmap();

    // Save image.
    image::save_buffer(
        &Path::new(OUTPUT_PATH),
        &texture_data,
        TEXTURE_DIMS.0 as u32,
        TEXTURE_DIMS.1 as u32,
        image::ExtendedColorType::Rgba8,
    )
    .unwrap();
    eprintln!("PNG file written to disc as \"{}\".", OUTPUT_PATH);

    eprintln!("Done.");
}

pub fn load_model_gltf(file_name: &str, device: &wgpu::Device, queue: &wgpu::Queue) -> Model {
    let gltf_text = std::fs::read_to_string(file_name).expect("wah");
    let gltf_cursor = Cursor::new(gltf_text);
    let gltf_reader = BufReader::new(gltf_cursor);
    let gltf = Gltf::from_reader(gltf_reader).expect("wah");

    // Load buffers
    let mut buffer_data = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                // if let Some(blob) = gltf.blob.as_deref() {
                //     buffer_data.push(blob.into());
                //     println!("Found a bin, saving");
                // };
            }
            gltf::buffer::Source::Uri(uri) => {
                let bin = std::fs::read(uri).expect("wah");
                buffer_data.push(bin);
            }
        }
    }

    // Load animations
    let mut animation_clips = Vec::new();
    for animation in gltf.animations() {
        for channel in animation.channels() {
            let reader = channel.reader(|buffer| Some(&buffer_data[buffer.index()]));
            let timestamps = if let Some(inputs) = reader.read_inputs() {
                match inputs {
                    gltf::accessor::Iter::Standard(times) => {
                        let times: Vec<f32> = times.collect();
                        println!("Time: {}", times.len());
                        dbg!(&times);
                        times
                    }
                    gltf::accessor::Iter::Sparse(_) => {
                        println!("Sparse keyframes not supported");
                        let times: Vec<f32> = Vec::new();
                        times
                    }
                }
            } else {
                println!("We got problems");
                let times: Vec<f32> = Vec::new();
                times
            };

            let keyframes = if let Some(outputs) = reader.read_outputs() {
                match outputs {
                    gltf::animation::util::ReadOutputs::Translations(translation) => {
                        let translation_vec = translation
                            .map(|tr| {
                                // println!("Translation:");
                                dbg!(&tr);
                                let vector: Vec<f32> = tr.into();
                                vector
                            })
                            .collect();
                        Keyframes::Translation(translation_vec)
                    }
                    other => Keyframes::Other, // gltf::animation::util::ReadOutputs::Rotations(_) => todo!(),
                                               // gltf::animation::util::ReadOutputs::Scales(_) => todo!(),
                                               // gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => todo!(),
                }
            } else {
                println!("We got problems");
                Keyframes::Other
            };

            animation_clips.push(AnimationClip {
                name: animation.name().unwrap_or("Default").to_string(),
                keyframes,
                timestamps,
            })
        }
    }

    // Load materials
    let mut materials = Vec::new();
    for material in gltf.materials() {
        println!("Looping thru materials");
        let pbr = material.pbr_metallic_roughness();
        let base_color_texture = &pbr.base_color_texture();
        let texture_source = &pbr
            .base_color_texture()
            .map(|tex| {
                // println!("Grabbing diffuse tex");
                // dbg!(&tex.texture().source());
                tex.texture().source().source()
            })
            .expect("texture");

        match texture_source {
            gltf::image::Source::View { view, mime_type } => {
                let diffuse_texture = texture::Texture::from_bytes(
                    device,
                    queue,
                    &buffer_data[view.buffer().index()],
                    file_name,
                );

                materials.push(model::Material {
                    name: material.name().unwrap_or("Default Material").to_string(),
                    diffuse_texture,
                });
            }
            gltf::image::Source::Uri { uri, mime_type } => {
                let diffuse_texture_bytes = std::fs::read(uri).expect("wah");
                let diffuse_texture =
                    texture::Texture::from_bytes(device, queue, &diffuse_texture_bytes, uri);

                materials.push(model::Material {
                    name: material.name().unwrap_or("Default Material").to_string(),
                    diffuse_texture,
                });
            }
        };
    }

    let mut meshes = Vec::new();

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            println!("Node {}", node.index());
            // dbg!(node);

            let mesh = node.mesh().expect("Got mesh");
            let primitives = mesh.primitives();
            primitives.for_each(|primitive| {
                // dbg!(primitive);

                let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

                let mut vertices = Vec::new();
                if let Some(vertex_attribute) = reader.read_positions() {
                    vertex_attribute.for_each(|vertex| {
                        // dbg!(vertex);
                        vertices.push(ModelVertex {
                            position: vertex,
                            tex_coords: Default::default(),
                            normal: Default::default(),
                        })
                    });
                }
                if let Some(normal_attribute) = reader.read_normals() {
                    let mut normal_index = 0;
                    normal_attribute.for_each(|normal| {
                        // dbg!(normal);
                        vertices[normal_index].normal = normal;

                        normal_index += 1;
                    });
                }
                if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
                    let mut tex_coord_index = 0;
                    tex_coord_attribute.for_each(|tex_coord| {
                        // dbg!(tex_coord);
                        vertices[tex_coord_index].tex_coords = tex_coord;

                        tex_coord_index += 1;
                    });
                }

                let mut indices = Vec::new();
                if let Some(indices_raw) = reader.read_indices() {
                    // dbg!(indices_raw);
                    indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
                }
                // dbg!(indices);

                // println!("{:#?}", &indices.expect("got indices").data_type());
                // println!("{:#?}", &indices.expect("got indices").index());
                // println!("{:#?}", &material);

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Vertex Buffer", file_name)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Index Buffer", file_name)),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                meshes.push(Mesh {
                    name: file_name.to_string(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: indices.len() as u32,
                    // material: m.mesh.material_id.unwrap_or(0),
                    material: 0,
                });
            });
        }
    }

    Model {
        meshes,
        materials,
        animations: animation_clips,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn render_model_to_image() {
        // TODO;
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(render_to_image());
        panic!("donezo");
    }
}
