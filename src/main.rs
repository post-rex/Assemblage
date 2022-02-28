#![feature(int_roundings)]

mod camera;
mod camera_controller;
mod state;
mod texture;
mod types;
mod voxels;

use camera::*;
use state::*;
use texture::Texture;
use voxels::voxel_scene::CHUNK_SIZE;

use glam::{IVec3, UVec3};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

use wgpu::util::DeviceExt;

use crate::camera_controller::CameraController;
use crate::types::vertex::Vertex;
use crate::voxels::voxel_scene::VoxelScene;

#[tokio::main]
async fn main() -> Result<(), ()> {
    env_logger::init(); // Tells WGPU to inform us of errors, rather than failing silently

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap(); // Create a window
    let mut scene = VoxelScene::new();
    let mut state = State::new(&window).await;

    generate_world(&mut scene, &mut state, UVec3::new(50, 1, 50)).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });

    Ok(())
}

pub async fn generate_world(scene: &mut VoxelScene, state: &mut State, size: UVec3) {
    state.render_passes.clear();
    state.add_render_pass();

    // Start timer
    let total_chunk_count = size.x * size.y * size.z;

    use std::time::Instant;
    let now = Instant::now();

    for x in 0..size.x {
        for y in 0..size.y {
            for z in 0..size.z {
                scene.initialize_chunk(&IVec3::new(x as i32, y as i32, z as i32));
            }
        }
    }

    scene.process_initialization_queue().await;

    let pass = state.render_passes.last_mut().unwrap();

    let mut combined_verts = Vec::new();
    let mut combined_indices = Vec::new();

    scene.chunks.iter_mut().for_each(|(position, chunk)| {
        let mesh = &mut chunk.mesh;
        let offset = combined_verts.len() as u32;

        mesh.vertices.par_iter_mut().for_each(|vert| {
            vert.position = [
                vert.position[0] + (position.x as f32 * CHUNK_SIZE as f32),
                vert.position[1] + (position.y as f32 * CHUNK_SIZE as f32),
                vert.position[2] + (position.z as f32 * CHUNK_SIZE as f32),
            ]
        });

        combined_verts.append(&mut mesh.vertices);

        mesh.indices.iter().for_each(|i| {
            combined_indices.push(i + offset);
        });
    });

    pass.set_vertices(&state.device, &mut combined_verts);
    pass.set_indices(&state.device, &mut combined_indices);

    // End timer
    let elapsed = now.elapsed();
    println!(
        "Generated {} chunks\nGeneration took {:.2?} per chunk\nWhich is {} chunks per second",
        total_chunk_count,
        elapsed / total_chunk_count,
        1.0 / (elapsed / total_chunk_count).as_secs_f32()
    );
}

pub struct RenderPassData {
    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    index_buffer: wgpu::Buffer,
    index_count: u32,

    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
}

impl RenderPassData {
    pub fn set_vertices(&mut self, device: &wgpu::Device, vertices: &Vec<Vertex>) {
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.vertex_count = vertices.len() as u32;
    }

    pub fn set_indices(&mut self, device: &wgpu::Device, indices: &Vec<u32>) {
        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        self.index_count = indices.len() as u32;
    }
}
