use chrono::Local;
use dirs;
use nannou::prelude::*;
use nannou_egui::{
    self,
    egui::{self, Color32, FontDefinitions, FontFamily},
    Egui,
};

use std::{cell::Cell, env, error::Error, fs};
use std::{path::PathBuf, str};

use framework::prelude::*;

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
    alert_text: String,
    recording: bool,
    recording_dir: Option<PathBuf>,
    recorded_frames: Cell<u32>,
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
        .size(
            sketch_config.gui_w.unwrap_or(350),
            sketch_config.gui_h.unwrap_or(350),
        )
        .view(view_gui::<S>)
        .raw_event(|_app, model: &mut AppModel<S>, event| {
            model.egui.handle_raw_event(event);
        })
        .build()
        .unwrap();

    set_window_position(app, main_window_id, 0, 0);
    set_window_position(app, gui_window_id, sketch_config.w * 2, 0);

    let egui = Egui::from_window(&app.window(gui_window_id).unwrap());

    let mut sketch_model = init_sketch_model();

    if let Some(values) = stored_controls(&sketch_config.name) {
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
        alert_text: "".into(),
        recording: false,
        recording_dir: frames_dir(sketch_config.name),
        recorded_frames: Cell::new(0),
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

    update_gui(
        app,
        model.main_window_id,
        model.sketch_config,
        &mut model.sketch_model,
        &mut model.alert_text,
        &mut model.recording,
        &mut model.recording_dir,
        &model.recorded_frames,
        &ctx,
    );
}

fn update_gui<S: SketchModel>(
    app: &App,
    main_window_id: window::Id,
    sketch_config: &SketchConfig,
    sketch_model: &mut S,
    alert_text: &mut String,
    recording: &mut bool,
    recording_dir: &mut Option<PathBuf>,
    recorded_frames: &Cell<u32>,
    ctx: &egui::Context,
) {
    let mut style = (*ctx.style()).clone();
    style.visuals.button_frame = true;
    style.visuals.widgets.inactive.bg_fill = Color32::from_gray(10);
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(10);
    style.spacing.slider_width = 160.0;
    ctx.set_style(style);
    setup_monospaced_fonts(ctx);

    egui::CentralPanel::default()
        .frame(
            egui::Frame::default()
                .fill(Color32::from_gray(3))
                .inner_margin(egui::Margin::same(16.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let main_window = app.window(main_window_id);

                ui.add(egui::Button::new("Save")).clicked().then(|| {
                    if let Some(window) = main_window {
                        capture_frame(
                            &window,
                            app,
                            sketch_config.name,
                            alert_text,
                        );
                    }
                });

                draw_pause_button(ui, alert_text);
                draw_reset_button(ui, alert_text);
                draw_clear_cache_button(ui, sketch_config.name, alert_text);
                draw_record_button(
                    ui,
                    sketch_config,
                    recording,
                    recording_dir,
                    recorded_frames,
                    alert_text,
                );
            });

            ui.separator();
            draw_sketch_controls(ui, sketch_model, sketch_config, alert_text);
            draw_alert_panel(ctx, alert_text);
        });
}

fn view<S>(
    app: &App,
    model: &AppModel<S>,
    frame: Frame,
    sketch_view_fn: fn(&App, &S, Frame),
) {
    let did_render = frame_controller::wrapped_view(
        app,
        &model.sketch_model,
        frame,
        sketch_view_fn,
    );

    if did_render && model.recording {
        let frame_count = model.recorded_frames.get();
        match app.window(model.main_window_id) {
            Some(window) => match &model.recording_dir {
                Some(path) => {
                    let filename = format!("frame-{:06}.png", frame_count);
                    window.capture_frame(path.join(filename));
                }
                None => error!("Unable to capture frame {}", frame_count),
            },
            None => panic!("Unable to attain app.window handle"),
        }
        model.recorded_frames.set(model.recorded_frames.get() + 1);
    }
}

fn view_gui<S>(_app: &App, model: &AppModel<S>, frame: Frame) {
    model.egui.draw_to_frame(&frame).unwrap();
}

fn persist_controls(
    sketch_name: &str,
    controls: &Controls,
) -> Result<PathBuf, Box<dyn Error>> {
    let path = controls_storage_path(sketch_name)
        .ok_or("Could not determine the configuration directory")?;
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    let json = serde_json::to_string_pretty(controls)?;
    fs::write(&path, json)?;
    Ok(path)
}

fn stored_controls(sketch_name: &str) -> Option<ControlValues> {
    let path = controls_storage_path(sketch_name)?;
    let bytes = fs::read(path).ok()?;
    let string = str::from_utf8(&bytes).ok()?;
    let controls = serde_json::from_str::<Controls>(string).ok()?;
    Some(controls.values().clone())
}

fn delete_stored_controls(sketch_name: &str) -> Result<(), Box<dyn Error>> {
    let path = controls_storage_path(sketch_name)
        .ok_or("Could not determine the configuration directory")?;
    if path.exists() {
        fs::remove_file(path)?;
        info!("Deleted controls for sketch: {}", sketch_name);
    } else {
        warn!("No stored controls found for sketch: {}", sketch_name);
    }
    Ok(())
}

fn controls_storage_path(sketch_name: &str) -> Option<PathBuf> {
    lattice_config_dir().map(|config_dir| {
        config_dir
            .join("Controls")
            .join(format!("{}_controls.json", sketch_name))
    })
}

fn frames_dir(sketch_name: &str) -> Option<PathBuf> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    lattice_config_dir().map(|config_dir| {
        config_dir
            .join("Captures")
            .join(sketch_name)
            .join(timestamp)
    })
}

fn lattice_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|config_dir| config_dir.join("Lattice"))
}

fn setup_monospaced_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts
        .families
        .insert(FontFamily::Monospace, vec!["Hack".to_owned()]);

    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();

    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(10.0, FontFamily::Monospace),
    );

    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(10.0, FontFamily::Monospace),
    );

    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(12.0, FontFamily::Monospace),
    );

    ctx.set_style(style);
}

fn capture_frame(
    window: &nannou::window::Window,
    app: &App,
    sketch_name: &str,
    alert_text: &mut String,
) {
    let filename = format!("{}-{}.png", sketch_name, uuid_5());
    let file_path = app.project_path().unwrap().join("images").join(&filename);

    window.capture_frame(file_path.clone());
    info!("Image saved to {:?}", file_path);
    *alert_text = format!("Image saved to {:?}", file_path);
}

fn draw_pause_button(ui: &mut egui::Ui, alert_text: &mut String) {
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
        *alert_text =
            (if next_is_paused { "Paused" } else { "Resumed" }).into();
    });
}

fn draw_reset_button(ui: &mut egui::Ui, alert_text: &mut String) {
    ui.add(egui::Button::new("Reset")).clicked().then(|| {
        frame_controller::reset_frame_count();
        info!("Frame count reset");
        *alert_text = "Reset".into()
    });
}

fn draw_clear_cache_button(
    ui: &mut egui::Ui,
    sketch_name: &str,
    alert_text: &mut String,
) {
    ui.add(egui::Button::new("Clear Cache")).clicked().then(|| {
        if let Err(e) = delete_stored_controls(sketch_name) {
            error!("Failed to clear controls cache: {}", e);
        } else {
            *alert_text = "Controls cache cleared".into();
        }
    });
}

fn draw_record_button(
    ui: &mut egui::Ui,
    sketch_config: &SketchConfig,
    recording: &mut bool,
    recording_dir: &mut Option<PathBuf>,
    recorded_frames: &Cell<u32>,
    alert_text: &mut String,
) {
    let is_recording = *recording;
    let button_label = if is_recording { "STOP" } else { "Record" };

    ui.add(egui::Button::new(button_label)).clicked().then(|| {
        let current_recording_dir = recording_dir.clone();
        if let Some(path) = current_recording_dir {
            *recording = !is_recording;
            info!("Recording: {}, path: {:?}", recording, path);

            if *recording {
                let message =
                    format!("Recording. Frames will be written to {:?}", path);
                *alert_text = message.into();
            } else {
                recorded_frames.set(0);
                let new_path = frames_dir(sketch_config.name).unwrap();
                *recording_dir = Some(new_path);

                let message = format!(
                    "Recording stopped. Frames are available at {:?}",
                    path
                );
                *alert_text = message.into();
            }
        } else {
            error!("Unable to access recording path");
        }
    });
}

fn draw_sketch_controls<S: SketchModel>(
    ui: &mut egui::Ui,
    sketch_model: &mut S,
    sketch_config: &SketchConfig,
    alert_text: &mut String,
) {
    if let Some(controls) = sketch_model.controls() {
        let any_changed = draw_controls(controls, ui);
        if any_changed {
            match persist_controls(sketch_config.name, controls) {
                Ok(path_buf) => {
                    *alert_text =
                        format!("Controls persisted at {:?}", path_buf);
                    debug!("Controls persisted at {:?}", path_buf);
                }
                Err(e) => {
                    error!("Failed to persist controls: {}", e);
                    *alert_text = "Failed to persist controls".into();
                }
            }
        }
    }
}

fn draw_alert_panel(ctx: &egui::Context, alert_text: &str) {
    egui::TopBottomPanel::bottom("alerts")
        .frame(
            egui::Frame::default()
                .fill(Color32::from_gray(2))
                .outer_margin(egui::Margin::same(6.0))
                .inner_margin(egui::Margin::same(4.0)),
        )
        .show_separator_line(false)
        .min_height(40.0)
        .show(ctx, |ui| {
            ui.colored_label(Color32::from_gray(180), alert_text);
        });
}
