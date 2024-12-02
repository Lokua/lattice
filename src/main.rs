use crate::framework::frame_controller;
use crate::framework::logger::init_logger;
use framework::util::{set_window_position, uuid_5};
use log::{info, warn};
use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};
use std::env;

pub mod framework;
mod sketches;

macro_rules! run_sketch {
    ($sketch:ident) => {{
        info!("Loading {}", sketches::$sketch::METADATA.display_name);
        frame_controller::init_controller(sketches::$sketch::METADATA.fps);
        nannou::app(model).update(update).view(view).run();
    }};
}

fn main() {
    init_logger();

    let args: Vec<String> = env::args().collect();
    let sketch_name = args.get(1).map(|s| s.as_str()).unwrap_or("template");

    match sketch_name {
        "template" => run_sketch!(template),
        "displacement_1" => run_sketch!(displacement_1),
        _ => {
            warn!("Sketch not found, running template");
            run_sketch!(template)
        }
    }
}

struct AppModel {
    main_window_id: window::Id,
    #[allow(dead_code)]
    gui_window_id: window::Id,
    egui: Egui,
    sketch_model: sketches::template::Model,
}

fn model(app: &App) -> AppModel {
    let w: i32 = 700;
    let h: i32 = 700;

    let main_window_id = app
        .new_window()
        .title(sketches::template::METADATA.display_name)
        .size(w as u32, h as u32)
        .build()
        .unwrap();

    let gui_window_id = app
        .new_window()
        .title(format!(
            "{} Controls",
            sketches::template::METADATA.display_name
        ))
        .size(w as u32 / 2, h as u32 / 2)
        .view(view_gui)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    set_window_position(app, main_window_id, 0, 0);
    set_window_position(app, gui_window_id, w * 2, 0);

    let egui = Egui::from_window(&app.window(gui_window_id).unwrap());

    let sketch_model = sketches::template::init_model();

    AppModel {
        main_window_id,
        gui_window_id,
        egui,
        sketch_model,
    }
}

fn update(app: &App, model: &mut AppModel, update: Update) {
    frame_controller::wrapped_update(
        app,
        &mut model.sketch_model,
        update,
        sketches::template::update,
    );

    model.egui.set_elapsed_time(update.since_start);
    let ctx = model.egui.begin_frame();

    egui::CentralPanel::default().show(&ctx, |ui| {
        if ui.button("Capture Frame").clicked() {
            if let Some(window) = app.window(model.main_window_id) {
                let filename = format!(
                    "{}-{}.png",
                    sketches::template::METADATA.name,
                    uuid_5()
                );
                let file_path =
                    app.project_path().unwrap().join("images").join(filename);
                window.capture_frame(file_path);
            }
        }

        // Sketch-specific controls
        // sketches::template::gui(&ctx, ui, &mut model.sketch_model);
    });
}

fn view(app: &App, model: &AppModel, frame: Frame) {
    frame_controller::wrapped_view(
        app,
        &model.sketch_model,
        frame,
        sketches::template::view,
    );
}

fn view_gui(_app: &App, model: &AppModel, frame: Frame) {
    model.egui.draw_to_frame(&frame).unwrap();
}

fn raw_window_event(
    _app: &App,
    model: &mut AppModel,
    event: &nannou::winit::event::WindowEvent,
) {
    model.egui.handle_raw_event(event);
}
