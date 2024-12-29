use gpu::GpuState;
use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "template_wgpu",
    display_name: "Template WGPU",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(360),
};

#[derive(SketchComponents)]
pub struct Model {
    #[allow(dead_code)]
    animation: Animation,
    controls: Controls,
    gpu: gpu::GpuState,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderParams {
    mode: u32,
    radius: f32,
}

pub fn init_model(app: &App, _wr: WindowRect) -> Model {
    let animation = Animation::new(SKETCH_CONFIG.bpm);

    let controls = Controls::with_previous(vec![
        Control::select("mode", "smooth", &["smooth", "step"]),
        Control::slider_norm("radius", 0.5),
    ]);

    let params = ShaderParams {
        mode: 0,
        radius: 0.0,
    };

    let shader = wgpu::include_wgsl!("./template_wgpu.wgsl");
    let gpu = GpuState::new(app, shader, &params);

    Model {
        animation,
        controls,
        gpu,
    }
}

pub fn update(app: &App, m: &mut Model, _update: Update) {
    let params = ShaderParams {
        mode: match m.controls.string("mode").as_str() {
            "smooth" => 0,
            "step" => 1,
            _ => unreachable!(),
        },
        radius: m.controls.float("radius"),
    };

    m.gpu.update_params(app, &params);
}

pub fn view(_app: &App, m: &Model, frame: Frame) {
    frame.clear(BLACK);
    m.gpu.render(&frame);
}
