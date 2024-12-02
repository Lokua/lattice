use crate::framework::frame_controller;
use crate::framework::logger::init_logger;
use framework::{
    sketch::SketchConfig,
    util::{set_window_position, uuid_5},
};
use log::{info, warn};
use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};
use std::env;

pub mod framework;
mod sketches;

macro_rules! run_sketch {
    ($sketch_module:ident) => {{
        info!(
            "Loading {}",
            sketches::$sketch_module::SKETCH_CONFIG.display_name
        );
        frame_controller::init_controller(
            sketches::$sketch_module::SKETCH_CONFIG.fps,
        );

        nannou::app(|app| {
            model(
                app,
                sketches::$sketch_module::init_model,
                &sketches::$sketch_module::SKETCH_CONFIG,
            )
        })
        .update(|app, model, nannou_update| {
            update(app, model, nannou_update, sketches::$sketch_module::update)
        })
        .view(|app, model, frame| {
            view(app, model, frame, sketches::$sketch_module::view)
        })
        .run();
    }};
}

fn main() {
    init_logger();

    let args: Vec<String> = env::args().collect();
    let sketch_name = args.get(1).map(|s| s.as_str()).unwrap_or("template");

    match sketch_name {
        "template" => run_sketch!(template),
        // "displacement_1" => run_sketch!(displacement_1),
        _ => {
            warn!("Sketch not found, running template");
            run_sketch!(template)
        }
    }
}

struct AppModel<S> {
    main_window_id: window::Id,
    #[allow(dead_code)]
    gui_window_id: window::Id,
    egui: Egui,
    sketch_model: S,
    sketch_config: &'static SketchConfig,
}

fn model<S: 'static>(
    app: &App,
    init_sketch_model: fn() -> S,
    sketch_config: &'static SketchConfig,
) -> AppModel<S> {
    let main_window_id = app
        .new_window()
        .title(sketch_config.display_name)
        .size(sketch_config.w, sketch_config.h)
        .build()
        .unwrap();

    let gui_window_id = app
        .new_window()
        .title(format!("{} Controls", sketch_config.display_name))
        .size(sketch_config.w / 2, sketch_config.h / 2)
        .view(view_gui::<S>)
        .raw_event(raw_window_event::<S>)
        .build()
        .unwrap();

    set_window_position(app, main_window_id, 0, 0);
    set_window_position(app, gui_window_id, (sketch_config.w * 2) as i32, 0);

    let egui = Egui::from_window(&app.window(gui_window_id).unwrap());

    let sketch_model = init_sketch_model();

    AppModel {
        main_window_id,
        gui_window_id,
        egui,
        sketch_model,
        sketch_config,
    }
}

fn update<S>(
    app: &App,
    model: &mut AppModel<S>,
    update: Update,
    sketch_update_fn: fn(&App, &mut S, Update),
    // sketch_gui_fn: fn(&egui::Context, &mut egui::Ui, &mut S),
) {
    frame_controller::wrapped_update(
        app,
        &mut model.sketch_model,
        update,
        sketch_update_fn,
    );

    model.egui.set_elapsed_time(update.since_start);
    let ctx = model.egui.begin_frame();

    egui::CentralPanel::default().show(&ctx, |ui| {
        if ui.button("Capture Frame").clicked() {
            if let Some(window) = app.window(model.main_window_id) {
                let filename =
                    format!("{}-{}.png", model.sketch_config.name, uuid_5());
                let file_path =
                    app.project_path().unwrap().join("images").join(filename);
                window.capture_frame(file_path);
            }
        }

        // Sketch-specific controls
        // sketch_gui_fn(&ctx, ui, &mut model.sketch_model);
    });
}

fn view<S>(
    app: &App,
    model: &AppModel<S>,
    frame: Frame,
    sketch_view_fn: fn(&App, &S, Frame),
) {
    frame_controller::wrapped_view(
        app,
        &model.sketch_model,
        frame,
        sketch_view_fn,
    );
}

fn view_gui<S>(_app: &App, model: &AppModel<S>, frame: Frame) {
    model.egui.draw_to_frame(&frame).unwrap();
}

fn raw_window_event<S>(
    _app: &App,
    model: &mut AppModel<S>,
    event: &nannou::winit::event::WindowEvent,
) {
    model.egui.handle_raw_event(event);
}
