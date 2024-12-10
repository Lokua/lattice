use std::{cell::Cell, error::Error, sync::mpsc, thread};
use std::{path::PathBuf, str};

use crate::framework::prelude::*;

#[derive(Default)]
pub struct RecordingState {
    pub active: bool,
    pub is_encoding: bool,
    pub recorded_frames: Cell<u32>,
    pub recording_dir: Option<PathBuf>,
    pub encoding_thread: Option<thread::JoinHandle<()>>,
    pub encoding_progress_rx: Option<mpsc::Receiver<EncodingMessage>>,
}

impl RecordingState {
    pub fn new(recording_dir: Option<PathBuf>) -> Self {
        Self {
            recording_dir,
            recorded_frames: Cell::new(0),
            ..Default::default()
        }
    }

    pub fn start_recording(
        &mut self,
        alert_text: &mut String,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(path) = &self.recording_dir {
            self.active = true;
            let message =
                format!("Recording. Frames will be written to {:?}", path);
            *alert_text = message;
            info!("Recording started, path: {:?}", path);
            Ok(())
        } else {
            Err("Unable to access recording path".into())
        }
    }

    pub fn stop_recording(
        &mut self,
        sketch_config: &SketchConfig,
        session_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        if !self.is_encoding {
            self.active = false;
            self.is_encoding = true;

            let (encoding_progress_tx, rx) = mpsc::channel();
            self.encoding_progress_rx = Some(rx);

            let path = self
                .recording_dir
                .as_ref()
                .ok_or("No recording directory")?
                .to_string_lossy()
                .into_owned();

            let output_path = video_output_path(session_id, sketch_config.name)
                .ok_or("Could not determine output path")?
                .to_string_lossy()
                .into_owned();

            let fps = sketch_config.fps;
            let total_frames = self.recorded_frames.get();

            info!("Preparing to encode. Output path: {}", output_path);
            debug!("Spawning encoding_thread");

            self.encoding_thread = Some(thread::spawn(move || {
                if let Err(e) = frames_to_video(
                    &path,
                    fps,
                    &output_path,
                    total_frames,
                    encoding_progress_tx,
                ) {
                    error!("Error in frames_to_video: {:?}", e);
                }
            }));

            Ok(())
        } else {
            Err("Already encoding".into())
        }
    }

    pub fn toggle_recording(
        &mut self,
        sketch_config: &SketchConfig,
        session_id: &str,
        alert_text: &mut String,
    ) -> Result<(), Box<dyn Error>> {
        if self.active {
            self.stop_recording(sketch_config, session_id)
        } else {
            self.start_recording(alert_text)
        }
    }
}

pub fn video_output_path(
    session_id: &str,
    sketch_name: &str,
) -> Option<PathBuf> {
    dirs::video_dir().map(|video_dir| {
        video_dir
            .join(format!("{}-{}", sketch_name, session_id))
            .with_extension("mp4")
    })
}
