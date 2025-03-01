use nannou::prelude::*;
use nannou_egui::Egui;
use std::cell::{Cell, Ref, RefCell};
use std::sync::mpsc;
use std::{env, str};

use super::prelude::*;
use super::tap_tempo::TapTempo;
use crate::framework::{frame_controller, prelude::*};

pub fn run() {
    init_logger();
    gui::init();

    {
        let mut registry = REGISTRY.write().unwrap();
        register_sketches!(registry, template);

        // ---------------------------------------------------------------------
        // MAIN
        // ---------------------------------------------------------------------
        register_legacy_sketches!(
            registry,
            blob,
            breakpoints,
            breakpoints_2,
            brutalism,
            displacement_2a,
            drop,
            drop_walk,
            floor_supervisor,
            flow_field_basic,
            heat_mask,
            interference,
            kalos,
            kalos_2,
            sand_lines,
            sierpinski_triangle,
            spiral,
            spiral_lines,
            wave_fract
        );

        // ---------------------------------------------------------------------
        // DEV
        // ---------------------------------------------------------------------
        register_legacy_sketches!(
            registry,
            animation_dev,
            audio_controls_dev,
            audio_dev,
            control_script_dev,
            cv_dev,
            midi_dev,
            multiple_sketches_dev,
            osc_dev,
            osc_transport_dev,
            responsive_dev,
            shader_to_texture_dev,
            wgpu_compute_dev,
            wgpu_dev
        );

        // ---------------------------------------------------------------------
        // GENUARY 2025
        // ---------------------------------------------------------------------
        register_legacy_sketches!(
            registry,
            g25_10_11_12,
            g25_13_triangle,
            g25_14_black_and_white,
            g25_18_wind,
            g25_19_op_art,
            g25_1_horiz_vert,
            g25_20_23_brutal_arch,
            g25_22_gradients_only,
            g25_26_symmetry,
            g25_2_layers,
            g25_5_isometric
        );

        // ---------------------------------------------------------------------
        // SCRATCH
        // ---------------------------------------------------------------------
        register_legacy_sketches!(
            registry,
            bos,
            chromatic_aberration,
            displacement_1,
            displacement_1a,
            displacement_2,
            lin_alg,
            lines,
            noise,
            perlin_loop,
            sand_line,
            shader_experiments,
            vertical,
            vertical_2,
            z_sim
        );

        // ---------------------------------------------------------------------
        // TEMPLATES
        // ---------------------------------------------------------------------
        register_legacy_sketches!(
            registry,
            basic_cube_shader_template,
            fullscreen_shader_template
        );

        registry.prepare();
    }

    nannou::app(model)
        .update(update)
        .view(view)
        .event(event)
        .run();
}

#[derive(Debug)]
pub enum AppEvent {
    AdvanceSingleFrame,
    Alert(String),
    CaptureFrame,
    ClearControlsCache,
    ClearNextFrame,
    ControlsChanged,
    MidiContinue,
    MidiStart,
    MidiStop,
    QueueRecord,
    Record,
    Reset,
    SwitchSketch(String),
    Tap,
    ToggleFullScreen,
    ToggleGuiFocus,
    ToggleMainFocus,
    TogglePerfMode(bool),
    TogglePlay,
    ToggleTapTempo(bool),
}

pub struct AppEventSender {
    tx: mpsc::Sender<AppEvent>,
}

impl AppEventSender {
    fn new(tx: mpsc::Sender<AppEvent>) -> Self {
        Self { tx }
    }

    pub fn alert(&self, message: impl Into<String>) {
        self.tx
            .send(AppEvent::Alert(message.into()))
            .expect("Failed to send alert event");
    }

    pub fn send(&self, event: AppEvent) {
        self.tx.send(event).expect("Failed to send event");
    }
}

pub type AppEventReceiver = mpsc::Receiver<AppEvent>;

struct AppModel {
    main_window_id: window::Id,
    gui_window_id: window::Id,
    egui: RefCell<Egui>,
    session_id: String,
    alert_text: String,
    clear_next_frame: Cell<bool>,
    tap_tempo: TapTempo,
    tap_tempo_enabled: bool,
    tap_tempo_bpm: f32,
    perf_mode: bool,
    recording_state: RecordingState,
    sketch: Box<dyn Sketch>,
    sketch_config: &'static SketchConfig,
    main_maximized: Cell<bool>,
    event_tx: AppEventSender,
    event_rx: AppEventReceiver,
}

impl AppModel {
    fn main_window<'a>(&self, app: &'a App) -> Option<Ref<'a, Window>> {
        app.window(self.main_window_id)
    }

    fn gui_window<'a>(&self, app: &'a App) -> Option<Ref<'a, Window>> {
        app.window(self.gui_window_id)
    }

    fn window_rect<'a>(&self, app: &'a App) -> Option<Rect> {
        self.main_window(app).map(|window| window.rect())
    }

    fn sketch_name(&self) -> String {
        self.sketch_config.name.to_string()
    }

    fn bpm(&self) -> f32 {
        ternary!(
            self.tap_tempo_enabled,
            self.tap_tempo_bpm,
            self.sketch_config.bpm
        )
    }

    fn on_app_event(&mut self, app: &App, event: AppEvent) {
        match event {
            AppEvent::AdvanceSingleFrame => {
                frame_controller::advance_single_frame();
            }
            AppEvent::Alert(text) => {
                self.alert_text = text;
            }
            AppEvent::CaptureFrame => {
                let filename =
                    format!("{}-{}.png", self.sketch_name(), uuid_5());

                let file_path =
                    lattice_project_root().join("images").join(&filename);

                self.main_window(app)
                    .unwrap()
                    .capture_frame(file_path.clone());

                let alert_text = format!("Image saved to {:?}", file_path);
                self.alert_text = alert_text.clone();
                info!("{}", alert_text);
            }
            AppEvent::ClearControlsCache => {
                if let Err(e) =
                    storage::delete_stored_controls(self.sketch_config.name)
                {
                    error!("Failed to clear controls cache: {}", e);
                } else {
                    self.alert_text = "Controls cache cleared".into();
                }
            }
            AppEvent::ClearNextFrame => {
                self.clear_next_frame.set(true);
            }
            AppEvent::ControlsChanged => {
                if frame_controller::is_paused()
                    && self.sketch_config.play_mode != PlayMode::ManualAdvance
                {
                    frame_controller::advance_single_frame();
                }

                match storage::persist_controls(
                    self.sketch_config.name,
                    self.sketch.controls_provided().unwrap(),
                ) {
                    Ok(path_buf) => {
                        let message =
                            format!("Controls persisted at {:?}", path_buf);
                        self.alert_text = message.clone();
                        trace!("{}", message);
                    }
                    Err(e) => {
                        let message =
                            format!("Failed to persist controls: {}", e);
                        self.alert_text = message.clone();
                        error!("{}", message);
                    }
                }
            }
            AppEvent::MidiStart | AppEvent::MidiContinue => {
                info!(
                    "Received MIDI Start/Continue message. \
                    Resetting frame count."
                );

                frame_controller::reset_frame_count();

                if self.recording_state.is_queued {
                    match self.recording_state.start_recording() {
                        Ok(message) => {
                            self.event_tx.alert(message);
                        }
                        Err(e) => {
                            let message =
                                format!("Failed to start recording: {}", e);
                            self.alert_text = message.clone();
                            error!("{}", message);
                        }
                    }
                }
            }
            AppEvent::MidiStop => {
                if self.recording_state.is_recording
                    && !self.recording_state.is_encoding
                {
                    match self
                        .recording_state
                        .stop_recording(self.sketch_config, &self.session_id)
                    {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed stop recording: {}", e);
                        }
                    }
                }
            }
            AppEvent::QueueRecord => {
                if self.recording_state.is_queued {
                    self.recording_state.is_queued = false;
                    self.alert_text = "".into();
                } else {
                    self.recording_state.is_queued = true;
                    self.alert_text =
                        "Recording queued. Awaiting MIDI Start message".into();
                }
            }
            AppEvent::Record => {
                match self
                    .recording_state
                    .toggle_recording(self.sketch_config, &self.session_id)
                {
                    Ok(message) => {
                        self.alert_text = message;
                    }
                    Err(e) => {
                        let message = format!("Recording error: {}", e);
                        self.alert_text = message.clone();
                        error!("{}", message);
                    }
                }
            }
            AppEvent::Reset => {
                frame_controller::reset_frame_count();
                self.alert_text = "Reset".into();
            }
            AppEvent::SwitchSketch(name) => {
                if self.sketch_config.name != name
                    && REGISTRY.read().unwrap().get(&name).is_some()
                {
                    self.switch_sketch(app, &name);
                }
            }
            AppEvent::Tap => {
                self.tap_tempo_bpm = self.tap_tempo.tap();
            }
            AppEvent::ToggleFullScreen => {
                let window = self.main_window(app).unwrap();
                if let Some(monitor) = window.current_monitor() {
                    let monitor_size = monitor.size();
                    let is_maximized = self.main_maximized.get();

                    if is_maximized {
                        window.set_inner_size_points(
                            self.sketch_config.w as f32,
                            self.sketch_config.h as f32,
                        );
                        self.main_maximized.set(false);
                    } else {
                        window.set_inner_size_pixels(
                            monitor_size.width,
                            monitor_size.height,
                        );
                        self.main_maximized.set(true);
                    }
                }
            }
            AppEvent::ToggleGuiFocus => {
                self.gui_window(app).unwrap().set_visible(true);
            }
            AppEvent::ToggleMainFocus => {
                self.main_window(app).unwrap().set_visible(true);
            }
            AppEvent::TogglePerfMode(ignore) => {
                self.perf_mode = ignore;
            }
            AppEvent::TogglePlay => {
                let next_is_paused = !frame_controller::is_paused();
                frame_controller::set_paused(next_is_paused);
                self.alert_text =
                    ternary!(next_is_paused, "Paused", "Resumed").into();
                info!("Paused: {}", next_is_paused);
            }
            AppEvent::ToggleTapTempo(tap_tempo_enabled) => {
                self.tap_tempo_enabled = tap_tempo_enabled;
                if tap_tempo_enabled {
                    self.tap_tempo_bpm = self.sketch_config.bpm;
                    self.alert_text = "Tap `Space` key to set BPM".into();
                } else {
                    self.alert_text =
                        "Tap tempo disabled. Sketch BPM has been restored."
                            .into();
                }
            }
        }
    }

    fn capture_recording_frame(&self, app: &App) {
        frame_controller::clear_force_render();

        if !self.recording_state.is_recording {
            return;
        }

        let frame_count = self.recording_state.recorded_frames.get();
        let window = self.main_window(app).unwrap();

        let recording_dir = match &self.recording_state.recording_dir {
            Some(path) => path,
            None => {
                error!("Unable to capture frame {}", frame_count);
                return;
            }
        };

        let filename = format!("frame-{:06}.png", frame_count);
        window.capture_frame(recording_dir.join(filename));

        self.recording_state.recorded_frames.set(frame_count + 1);
    }

    fn switch_sketch(&mut self, app: &App, name: &str) {
        let registry = REGISTRY.read().unwrap();
        let sketch_info = registry.get(name).unwrap();

        let rect = self.window_rect(app).unwrap();
        let sketch = (sketch_info.factory)(app, rect);

        self.sketch = sketch;
        self.sketch_config = sketch_info.config;
        self.session_id = uuid_5();
        self.clear_next_frame.set(true);

        self.init_sketch_environment(app);

        self.alert_text =
            format!("Switched to {}", sketch_info.config.display_name);
    }

    fn init_sketch_environment(&mut self, app: &App) {
        self.recording_state = RecordingState::new(frames_dir(
            &self.session_id,
            &self.sketch_config.name,
        ));

        self.main_window(app).map(|window| {
            window.set_title(&self.sketch_config.display_name);
            if !self.perf_mode {
                set_window_position(app, self.main_window_id, 0, 0);
                set_window_size(
                    window.winit_window(),
                    self.sketch_config.w,
                    self.sketch_config.h,
                );
            }
        });

        let (gui_w, gui_h) =
            gui::calculate_gui_dimensions(self.sketch.controls_provided());

        self.gui_window(app).map(|gui_window| {
            gui_window.set_title(&format!(
                "{} Controls",
                self.sketch_config.display_name
            ));

            if !self.perf_mode {
                set_window_position(
                    app,
                    self.gui_window_id,
                    self.sketch_config.w * 2,
                    0,
                );
            }

            set_window_size(
                gui_window.winit_window(),
                self.sketch_config.gui_w.unwrap_or(gui_w) as i32,
                self.sketch_config.gui_h.unwrap_or(gui_h) as i32,
            );
        });

        frame_controller::ensure_controller(self.sketch_config.fps);

        if self.sketch_config.play_mode != PlayMode::Loop {
            frame_controller::set_paused(true);
        }

        if let (Some(values), Some(controls)) = (
            storage::stored_controls(self.sketch_config.name),
            self.sketch.controls_provided(),
        ) {
            for (name, value) in values.into_iter() {
                controls.update_value(&name, value);
            }
            info!("Controls restored");
        }
    }
}

fn model(app: &App) -> AppModel {
    let args: Vec<String> = env::args().collect();
    let initial_sketch = args
        .get(1)
        .map(|s| s.to_string())
        .unwrap_or_else(|| "template".to_string());

    let registry = REGISTRY.read().unwrap();
    let sketch_info = registry
        .get(&initial_sketch)
        .unwrap_or_else(|| panic!("Sketch not found: {}", initial_sketch));

    app.set_fullscreen_on_shortcut(false);

    let main_window_id = app.new_window().build().unwrap();

    let window_rect = app
        .window(main_window_id)
        .expect("Unable to get window")
        .rect();

    let sketch = (sketch_info.factory)(app, window_rect);

    let gui_window_id = app
        .new_window()
        .view(view_gui)
        .raw_event(|_app, model: &mut AppModel, event| {
            model.egui.get_mut().handle_raw_event(event);
        })
        .build()
        .unwrap();

    let egui =
        RefCell::new(Egui::from_window(&app.window(gui_window_id).unwrap()));

    let (raw_event_tx, event_rx) = mpsc::channel();
    let midi_tx = raw_event_tx.clone();

    midi::on_message(
        midi::ConnectionType::GlobalStartStop,
        crate::config::MIDI_CLOCK_PORT,
        move |message| match message[0] {
            START => midi_tx.send(AppEvent::MidiStart).unwrap(),
            CONTINUE => midi_tx.send(AppEvent::MidiContinue).unwrap(),
            STOP => midi_tx.send(AppEvent::MidiStop).unwrap(),
            _ => {}
        },
    )
    .expect(&format!(
        "Failed to initialize {:?} MIDI connection",
        midi::ConnectionType::GlobalStartStop
    ));

    let mut model = AppModel {
        main_window_id,
        gui_window_id,
        egui,
        session_id: uuid_5(),
        alert_text: String::new(),
        clear_next_frame: Cell::new(true),
        perf_mode: false,
        tap_tempo: TapTempo::new(),
        tap_tempo_enabled: false,
        tap_tempo_bpm: 134.0,
        recording_state: RecordingState::new(frames_dir("", "")),
        sketch,
        sketch_config: sketch_info.config,
        main_maximized: Cell::new(false),
        event_tx: AppEventSender::new(raw_event_tx),
        event_rx,
    };

    model.init_sketch_environment(app);

    model
}

fn update(app: &App, model: &mut AppModel, update: Update) {
    model.egui.borrow_mut().set_elapsed_time(update.since_start);

    {
        let mut egui = model.egui.borrow_mut();
        let ctx = egui.begin_frame();
        let bpm = model.bpm();
        gui::update(
            &model.sketch_config,
            model.sketch.controls_provided(),
            &mut model.alert_text,
            &mut model.perf_mode,
            &mut model.tap_tempo_enabled,
            bpm,
            &mut model.recording_state,
            &model.event_tx,
            &ctx,
        );
    }

    while let Ok(event) = model.event_rx.try_recv() {
        model.on_app_event(app, event);
    }

    model.main_window(app).map(|window| {
        let rect = window.rect();
        model.sketch.set_window_rect(rect);
    });

    let bpm = model.bpm();

    frame_controller::wrapped_update(
        app,
        &mut model.sketch,
        update,
        |app, sketch, update| {
            sketch.update(
                app,
                update,
                &LatticeContext {
                    bpm,
                    window_rect: None,
                },
            )
        },
    );

    if model.recording_state.is_encoding {
        model.recording_state.on_encoding_message(
            &model.sketch_config,
            &mut model.session_id,
            &model.event_tx,
        );
    }
}

/// Note: this is shared between main and gui windows
fn event(app: &App, model: &mut AppModel, event: Event) {
    match event {
        Event::WindowEvent {
            simple: Some(KeyPressed(key)),
            ..
        } => {
            let logo_pressed = app.keys.mods.logo();
            let shift_pressed = app.keys.mods.shift();
            let has_no_modifiers = !app.keys.mods.alt()
                && !app.keys.mods.ctrl()
                && !shift_pressed
                && !logo_pressed;

            match key {
                Key::Space => {
                    model.event_tx.send(AppEvent::Tap);
                }
                // A
                Key::A if has_no_modifiers => {
                    model.event_tx.send(AppEvent::AdvanceSingleFrame);
                }
                // Cmd + F
                Key::F if logo_pressed => {
                    model.event_tx.send(AppEvent::ToggleFullScreen);
                }
                // Cmd + Shift + F
                // Key::F if logo_pressed && shift_pressed => {
                //     model.main_window(app).unwrap().set_fullscreen_with(Some(
                //         Fullscreen::Borderless(None),
                //     ));
                // }
                // Cmd + G
                Key::G if logo_pressed => {
                    model.event_tx.send(AppEvent::ToggleGuiFocus);
                }
                // Cmd + M
                Key::M if logo_pressed => {
                    model.event_tx.send(AppEvent::ToggleMainFocus);
                }
                // R
                Key::R if has_no_modifiers => {
                    model.event_tx.send(AppEvent::Reset);
                }
                // S
                Key::S if has_no_modifiers => {
                    model.event_tx.send(AppEvent::CaptureFrame);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn view(app: &App, model: &AppModel, frame: Frame) {
    if model.clear_next_frame.get() {
        frame.clear(model.sketch.clear_color());
        model.clear_next_frame.set(false);
    }

    let bpm = model.bpm();
    let did_render = frame_controller::wrapped_view(
        app,
        &model.sketch,
        frame,
        |app, sketch, frame| {
            sketch.view(
                app,
                frame,
                &LatticeContext {
                    bpm,
                    window_rect: None,
                },
            )
        },
    );

    if did_render {
        model.capture_recording_frame(app);
    }
}

fn view_gui(_app: &App, model: &AppModel, frame: Frame) {
    model.egui.borrow().draw_to_frame(&frame).unwrap();
}
