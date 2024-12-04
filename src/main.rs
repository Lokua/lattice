use dirs;
use nannou::prelude::*;
use nannou_egui::{
    self,
    egui::{self, Color32},
    Egui,
};
use std::{env, error::Error, fs};
use std::{path::PathBuf, str};

use framework::{
    controls::{draw_controls, ControlValues, Controls},
    frame_controller,
    logging::*,
    sketch::{SketchConfig, SketchModel},
    util::{set_window_position, uuid_5},
};

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
            update::<sketches::$sketch_module::Model>(
                app,
                model,
                nannou_update,
                sketches::$sketch_module::update,
            )
        })
        .view(|app, model, frame| {
            view::<sketches::$sketch_module::Model>(
                app,
                model,
                frame,
                sketches::$sketch_module::view,
            )
        })
        .run();
    }};
}

fn main() {
    init_logger();

    let args: Vec<String> = env::args().collect();
    let sketch_name = args.get(1).map(|s| s.as_str()).unwrap_or("template");

    match sketch_name {
        "displacement_1" => run_sketch!(displacement_1),
        "displacement_2" => run_sketch!(displacement_2),
        "template" => run_sketch!(template),
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

fn model<S: SketchModel + 'static>(
    app: &App,
    init_sketch_model: fn() -> S,
    sketch_config: &'static SketchConfig,
) -> AppModel<S> {
    let w = sketch_config.w as u32;
    let h = sketch_config.h as u32;

    let main_window_id = app
        .new_window()
        .title(sketch_config.display_name)
        .size(w, h)
        .build()
        .unwrap();

    let gui_window_id = app
        .new_window()
        .title(format!("{} Controls", sketch_config.display_name))
        .size(350, 350)
        .view(view_gui::<S>)
        .raw_event(raw_window_event::<S>)
        .build()
        .unwrap();

    set_window_position(app, main_window_id, 0, 0);
    set_window_position(app, gui_window_id, sketch_config.w * 2, 0);

    let egui = Egui::from_window(&app.window(gui_window_id).unwrap());

    let mut sketch_model = init_sketch_model();

    if let Some(values) = get_stored_controls(&sketch_config.name) {
        if let Some(controls) = sketch_model.controls() {
            for (name, value) in values.into_iter() {
                controls.update_value(&name, value);
            }
            info!("Controls restored")
        }
    }

    AppModel {
        main_window_id,
        gui_window_id,
        egui,
        sketch_model,
        sketch_config,
    }
}

fn update<S: SketchModel>(
    app: &App,
    model: &mut AppModel<S>,
    update: Update,
    sketch_update_fn: fn(&App, &mut S, Update),
) {
    frame_controller::wrapped_update(
        app,
        &mut model.sketch_model,
        update,
        sketch_update_fn,
    );

    model.egui.set_elapsed_time(update.since_start);
    let ctx = model.egui.begin_frame();

    let mut style = (*ctx.style()).clone();
    style.visuals.button_frame = true;
    style.visuals.widgets.inactive.bg_fill = Color32::from_gray(10);
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(10);
    // Unfortunately padding also impacts the "text-input" next to sliders.
    // style.spacing.button_padding = egui::Vec2::new(12.0, 4.0);
    style.spacing.slider_width = 160.0;
    // nannou_egui is behind
    // style.spacing.slider_rail_height = 4.0;
    ctx.set_style(style);

    egui::CentralPanel::default()
        .frame(
            egui::Frame::default()
                .fill(Color32::from_gray(3))
                .inner_margin(egui::Margin::same(16.0)),
        )
        .show(&ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Button::new("Capture Frame")).clicked().then(
                    || {
                        if let Some(window) = app.window(model.main_window_id) {
                            let filename = format!(
                                "{}-{}.png",
                                model.sketch_config.name,
                                uuid_5()
                            );

                            let file_path = app
                                .project_path()
                                .unwrap()
                                .join("images")
                                .join(filename);

                            window.capture_frame(file_path.clone());
                            info!("Image saved to {:?}", file_path);
                        }
                    },
                );

                ui.add(egui::Button::new(if frame_controller::is_paused() {
                    "Resume"
                } else {
                    "Pause"
                }))
                .clicked()
                .then(|| {
                    let next_is_paused = !frame_controller::is_paused();
                    frame_controller::set_paused(next_is_paused);
                    info!("Paused: {}", next_is_paused);
                });

                ui.add(egui::Button::new("Clear Cache"))
                    .clicked()
                    .then(|| delete_stored_controls(model.sketch_config.name));
            });

            ui.separator();

            if let Some(controls) = model.sketch_model.controls() {
                let any_changed = draw_controls(controls, ui);
                if any_changed {
                    match persist_controls(model.sketch_config.name, controls) {
                        Ok(_) => debug!("Controls persisted"),
                        Err(e) => error!("Failed to persist controls: {}", e),
                    }
                }
            }
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

fn persist_controls(
    sketch_name: &str,
    controls: &Controls,
) -> Result<(), Box<dyn Error>> {
    let path = get_controls_storage_path(sketch_name)
        .ok_or("Could not determine the configuration directory")?;
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    let json = serde_json::to_string_pretty(controls)?;
    fs::write(&path, json)?;
    Ok(())
}

fn get_stored_controls(sketch_name: &str) -> Option<ControlValues> {
    let path = get_controls_storage_path(sketch_name)?;
    let bytes = fs::read(path).ok()?;
    let string = str::from_utf8(&bytes).ok()?;
    let controls = serde_json::from_str::<Controls>(string).ok()?;
    Some(controls.get_values().clone())
}

fn delete_stored_controls(sketch_name: &str) -> Result<(), Box<dyn Error>> {
    let path = get_controls_storage_path(sketch_name)
        .ok_or("Could not determine the configuration directory")?;
    if path.exists() {
        fs::remove_file(path)?;
        info!("Deleted controls for sketch: {}", sketch_name);
    } else {
        warn!("No stored controls found for sketch: {}", sketch_name);
    }
    Ok(())
}

fn get_controls_storage_path(sketch_name: &str) -> Option<PathBuf> {
    dirs::config_dir().map(|config_dir| {
        config_dir
            .join("Lattice")
            .join(format!("{}_controls.json", sketch_name))
    })
}
