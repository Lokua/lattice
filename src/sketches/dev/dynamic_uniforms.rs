use nannou::prelude::*;

use crate::framework::prelude::*;

pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
    name: "dynamic_uniforms",
    display_name: "Dynamic Uniforms Dev",
    play_mode: PlayMode::Loop,
    fps: 60.0,
    bpm: 134.0,
    w: 700,
    h: 700,
    gui_w: None,
    gui_h: Some(600),
};

#[derive(SketchComponents)]
pub struct DynamicUniformsDev {
    hub: ControlHub<Timing>,
    gpu: gpu::GpuState<gpu::BasicPositionVertex>,
}

#[uniforms(banks = 8)]
struct ShaderParams {}

pub fn init(app: &App, ctx: &LatticeContext) -> DynamicUniformsDev {
    let wr = ctx.window_rect();
    let hub = ControlHub::from_path(
        to_absolute_path(file!(), "./dynamic_uniforms.yaml"),
        Timing::new(ctx.bpm()),
    );

    let params = ShaderParams::default();

    let gpu = gpu::GpuState::new_fullscreen(
        app,
        wr.resolution_u32(),
        to_absolute_path(file!(), "./dynamic_uniforms.wgsl"),
        &params,
        true,
    );

    DynamicUniformsDev { hub, gpu }
}

impl Sketch for DynamicUniformsDev {
    fn update(&mut self, app: &App, _update: Update, ctx: &LatticeContext) {
        self.hub.update();
        let wr = ctx.window_rect();
        let params = ShaderParams::from((&wr, &self.hub));
        self.gpu.update_params(app, wr.resolution_u32(), &params);
    }

    fn view(&self, _app: &App, frame: Frame, _ctx: &LatticeContext) {
        frame.clear(BLACK);
        self.gpu.render(&frame);
    }
}
