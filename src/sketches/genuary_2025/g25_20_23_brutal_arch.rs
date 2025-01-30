use bevy_reflect::Reflect;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "g25_20_23_brutal_arch",
    display_name:
        "Genuary 20, 23 | Generative Architecture, Inspired by Brutalism",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(360),
};

const BACKGROUND: f32 = 0.0;
const FOREGROUND: f32 = 1.0;

#[derive(SketchComponents)]
pub struct Model {
    controls: ControlScript<FrameTiming>,
    wr: WindowRect,
    gpu: gpu::GpuState<Vertex>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Reflect)]
struct Vertex {
    position: [f32; 3],
    layer: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderParams {
    // w, h, ..unused
    resolution: [f32; 4],

    // rot_x, rot_y, rot_z, scale
    a: [f32; 4],

    // unused
    b: [f32; 4],
}

pub fn init_model(app: &App, wr: WindowRect) -> Model {
    let controls = ControlScript::new(
        to_absolute_path(file!(), "g25_20_23_brutal_arch.yaml"),
        FrameTiming::new(SKETCH_CONFIG.bpm),
    );

    let params = ShaderParams {
        resolution: [0.0; 4],
        a: [0.0; 4],
        b: [0.0; 4],
    };

    // 6 vertices for the background +
    // 6 vertices * 6 faces for the foreground = 42
    let vertices = vec![
        Vertex {
            position: [0.0; 3],
            layer: BACKGROUND
        };
        42
    ];

    let gpu = gpu::GpuState::new(
        app,
        to_absolute_path(file!(), "g25_20_23_brutal_arch.wgsl"),
        &params,
        Some(&vertices),
        wgpu::PrimitiveTopology::TriangleList,
        Some(wgpu::BlendState::ALPHA_BLENDING),
        true,
        true,
    );

    Model { controls, wr, gpu }
}

pub fn update(app: &App, m: &mut Model, _update: Update) {
    m.controls.update();

    let params = ShaderParams {
        resolution: [m.wr.w(), m.wr.h(), 0.0, 0.0],
        a: [
            m.controls.get("rot_x"),
            m.controls.get("rot_y"),
            m.controls.get("rot_z"),
            m.controls.get("z_offset"),
        ],
        b: [
            m.controls.get("scale"),
            m.controls.get("b2"),
            m.controls.get("b3"),
            m.controls.get("b4"),
        ],
    };

    let mut vertices = Vec::with_capacity(42);
    vertices.extend(create_fullscreen_quad());
    vertices.extend(create_cube());

    m.gpu.update(app, &params, &vertices);
}

pub fn view(_app: &App, m: &Model, frame: Frame) {
    frame.clear(BLACK);
    m.gpu.render(&frame);
}

fn create_fullscreen_quad() -> Vec<Vertex> {
    vec![
        Vertex {
            // Bottom-left
            position: [-1.0, -1.0, 0.0],
            layer: BACKGROUND,
        },
        Vertex {
            // Bottom-right
            position: [1.0, -1.0, 0.0],
            layer: BACKGROUND,
        },
        Vertex {
            // Top-right
            position: [1.0, 1.0, 0.0],
            layer: BACKGROUND,
        },
        Vertex {
            // Bottom-left
            position: [-1.0, -1.0, 0.0],
            layer: BACKGROUND,
        },
        Vertex {
            // Top-right
            position: [1.0, 1.0, 0.0],
            layer: BACKGROUND,
        },
        Vertex {
            // Top-left
            position: [-1.0, 1.0, 0.0],
            layer: BACKGROUND,
        },
    ]
}

fn create_cube() -> Vec<Vertex> {
    let front_face = vec![
        Vertex {
            position: [-0.5, -0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, -0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [-0.5, -0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [-0.5, 0.5, 0.5],
            layer: FOREGROUND,
        },
    ];

    let back_face = vec![
        Vertex {
            position: [-0.5, -0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [-0.5, 0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, 0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [-0.5, -0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, 0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, -0.5, -0.5],
            layer: FOREGROUND,
        },
    ];

    let top_face = vec![
        Vertex {
            position: [-0.5, 0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [-0.5, 0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [-0.5, 0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, 0.5, -0.5],
            layer: FOREGROUND,
        },
    ];

    let bottom_face = vec![
        Vertex {
            position: [-0.5, -0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, -0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, -0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [-0.5, -0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, -0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [-0.5, -0.5, 0.5],
            layer: FOREGROUND,
        },
    ];

    let right_face = vec![
        Vertex {
            position: [0.5, -0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, 0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, -0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [0.5, -0.5, 0.5],
            layer: FOREGROUND,
        },
    ];

    let left_face = vec![
        Vertex {
            position: [-0.5, -0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [-0.5, -0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [-0.5, 0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [-0.5, -0.5, -0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [-0.5, 0.5, 0.5],
            layer: FOREGROUND,
        },
        Vertex {
            position: [-0.5, 0.5, -0.5],
            layer: FOREGROUND,
        },
    ];

    // 6 vertices * 6 faces
    let mut vertices = Vec::with_capacity(36);

    vertices.extend(front_face);
    vertices.extend(back_face);
    vertices.extend(top_face);
    vertices.extend(bottom_face);
    vertices.extend(right_face);
    vertices.extend(left_face);

    vertices
}
