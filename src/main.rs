use dirs;
use nannou::prelude::*;
use nannou_egui::{
    self,
    egui::{self, Color32, FontDefinitions, FontFamily},
    Egui,
};
use std::{cell::Cell, env, error::Error, fs, sync::mpsc, thread};
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
        frame_controller::ensure_controller(
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
        "displacement_2a" => run_sketch!(displacement_2a),
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
    session_id: String,
    alert_text: String,
    recording: bool,
    recording_dir: Option<PathBuf>,
    recorded_frames: Cell<u32>,
    is_encoding: bool,
    encoding_thread: Option<thread::JoinHandle<()>>,
    encoding_progress_rx: Option<mpsc::Receiver<EncodingMessage>>,
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
    let mut sketch_model = init_sketch_model();

    let main_window_id = app
        .new_window()
        .title(sketch_config.display_name)
        .size(w, h)
        .build()
        .unwrap();

    let (gui_w, gui_h) = calculate_gui_dimensions(sketch_model.controls());

    let gui_window_id = app
        .new_window()
        .title(format!("{} Controls", sketch_config.display_name))
        .size(
            sketch_config.gui_w.unwrap_or(gui_w),
            sketch_config.gui_h.unwrap_or(gui_h),
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

    if let Some(values) = stored_controls(&sketch_config.name) {
        if let Some(controls) = sketch_model.controls() {
            for (name, value) in values.into_iter() {
                controls.update_value(&name, value);
            }
            info!("Controls restored")
        }
    }

    let session_id = generate_session_id();
    let recording_dir = frames_dir(&session_id, sketch_config.name);
    AppModel {
        main_window_id,
        gui_window_id,
        egui,
        session_id,
        alert_text: "".into(),
        recording: false,
        recording_dir,
        recorded_frames: Cell::new(0),
        is_encoding: false,
        encoding_thread: None,
        encoding_progress_rx: None,
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
        &mut model.session_id,
        model.sketch_config,
        &mut model.sketch_model,
        &mut model.alert_text,
        &mut model.recording,
        &mut model.recording_dir,
        &model.recorded_frames,
        &mut model.is_encoding,
        &mut model.encoding_thread,
        &mut model.encoding_progress_rx,
        &ctx,
    );
}

fn update_gui<S: SketchModel>(
    app: &App,
    main_window_id: window::Id,
    session_id: &mut String,
    sketch_config: &SketchConfig,
    sketch_model: &mut S,
    alert_text: &mut String,
    recording: &mut bool,
    recording_dir: &mut Option<PathBuf>,
    recorded_frames: &Cell<u32>,
    is_encoding: &mut bool,
    encoding_thread: &mut Option<thread::JoinHandle<()>>,
    encoding_progress_rx: &mut Option<mpsc::Receiver<EncodingMessage>>,
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
                    session_id,
                    recording,
                    recording_dir,
                    recorded_frames,
                    is_encoding,
                    encoding_thread,
                    encoding_progress_rx,
                    alert_text,
                );

                draw_avg_fps(ui);
            });

            ui.separator();
            draw_sketch_controls(ui, sketch_model, sketch_config, alert_text);
            draw_alert_panel(ctx, alert_text);
        });

    if let Some(rx) = encoding_progress_rx.take() {
        while let Ok(message) = rx.try_recv() {
            match message {
                EncodingMessage::Progress(progress) => {
                    let percentage = (progress * 100.0).round();
                    debug!("rx progress: {}%", percentage);
                    *alert_text =
                        format!("Encoding progress: {}%", percentage).into();
                }
                EncodingMessage::Complete => {
                    info!("Encoding complete");
                    *is_encoding = false;
                    *encoding_progress_rx = None;
                    let output_path =
                        video_output_path(session_id, sketch_config.name)
                            .unwrap()
                            .to_string_lossy()
                            .into_owned();
                    *alert_text = format!(
                        "Encoding complete. Video path {}",
                        output_path
                    )
                    .into();
                    *session_id = generate_session_id();
                    recorded_frames.set(0);
                    if let Some(new_path) =
                        frames_dir(session_id, sketch_config.name)
                    {
                        *recording_dir = Some(new_path);
                    }
                }
                EncodingMessage::Error(error) => {
                    error!("Received child process error: {}", error);
                    *alert_text = format!("Received encoding error: {}", error);
                }
            }
        }
        *encoding_progress_rx = Some(rx);
    }
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

fn generate_session_id() -> String {
    uuid(5)
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

fn frames_dir(session_id: &str, sketch_name: &str) -> Option<PathBuf> {
    lattice_config_dir().map(|config_dir| {
        config_dir
            .join("Captures")
            .join(sketch_name)
            .join(session_id)
    })
}

fn lattice_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|config_dir| config_dir.join("Lattice"))
}

fn video_output_path(session_id: &str, sketch_name: &str) -> Option<PathBuf> {
    dirs::video_dir().map(|video_dir| {
        video_dir
            .join(format!("{}-{}", sketch_name, session_id))
            .with_extension("mp4")
    })
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

// I suck at math. Can't figure this out. EGUI controls don't seem to stack linearly.
// The more controls, the more the empty space grows between the last control and alert panel.
// Stick with per-sketch heights for now.
fn calculate_gui_dimensions(controls: Option<&mut Controls>) -> (u32, u32) {
    const HEADER_HEIGHT: u32 = 40;
    const ALERT_HEIGHT: u32 = 40;
    const CONTROL_HEIGHT: u32 = 26;
    const THRESHOLD: u32 = 5;
    const MIN_FINAL_GAP: u32 = 4;

    let controls_height = controls.map_or(0, |controls| {
        let count = controls.get_controls().len() as u32;
        let reduced_height = (CONTROL_HEIGHT as f32 * 0.95) as u32;
        let base = THRESHOLD * reduced_height;
        let remaining = count - THRESHOLD;
        base + (remaining * reduced_height)
    });

    let height = HEADER_HEIGHT + controls_height + MIN_FINAL_GAP + ALERT_HEIGHT;

    (350, height)
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
    session_id: &str,
    recording: &mut bool,
    recording_dir: &mut Option<PathBuf>,
    recorded_frames: &Cell<u32>,
    is_encoding: &mut bool,
    encoding_thread: &mut Option<thread::JoinHandle<()>>,
    encoding_progress_rx: &mut Option<mpsc::Receiver<EncodingMessage>>,
    alert_text: &mut String,
) {
    let button_label = if *recording {
        "STOP"
    } else if *is_encoding {
        "Encoding"
    } else {
        "Record"
    };

    ui.add_enabled(!*is_encoding, egui::Button::new(button_label))
        .clicked()
        .then(|| {
            let current_recording_dir = recording_dir.clone();
            if let Some(path) = current_recording_dir {
                *recording = !*recording;
                info!("Recording: {}, path: {:?}", recording, path);

                if *recording {
                    let message = format!(
                        "Recording. Frames will be written to {:?}",
                        path
                    );
                    *alert_text = message.into();
                } else {
                    if !*is_encoding {
                        *is_encoding = true;
                        let (encoding_progress_tx, rx): (
                            mpsc::Sender<EncodingMessage>,
                            mpsc::Receiver<EncodingMessage>,
                        ) = mpsc::channel();
                        *encoding_progress_rx = Some(rx);
                        let path_str = path.to_string_lossy().into_owned();
                        let fps = sketch_config.fps;
                        let output_path =
                            video_output_path(session_id, sketch_config.name)
                                .unwrap()
                                .to_string_lossy()
                                .into_owned();
                        info!(
                            "Preparing to encode. Output path: {}",
                            output_path
                        );
                        let total_frames = recorded_frames.get().clone();
                        debug!("Spawning encoding_thread");
                        *encoding_thread = Some(thread::spawn(move || {
                            if let Err(e) = frames_to_video(
                                &path_str,
                                fps,
                                &output_path,
                                total_frames,
                                encoding_progress_tx,
                            ) {
                                error!("Error in frames_to_video: {:?}", e);
                            }
                        }));
                    }
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
                    trace!("Controls persisted at {:?}", path_buf);
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

fn draw_avg_fps(ui: &mut egui::Ui) {
    let avg_fps = frame_controller::average_fps();
    ui.label("FPS:");
    ui.colored_label(Color32::from_rgb(0, 255, 0), format!("{:.1}", avg_fps));
}
