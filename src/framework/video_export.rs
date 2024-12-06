use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

use super::prelude::*;

pub enum EncodingMessage {
    /// Progress updates as a percentage (0.0 to 1.0)
    Progress(f32),
    /// Indicates encoding has completed
    Complete,
    Error(String),
}

pub fn frames_to_video(
    frame_dir: &str,
    fps: f64,
    output_path: &str,
    total_frames: u32,
    progress_sender: mpsc::Sender<EncodingMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    let process = Command::new("ffmpeg")
        .args([
            "-n", // Don't overwrite
            "-loglevel",
            "level+info",
            "-framerate",
            &fps.to_string(),
            "-i",
            &format!("{}/frame-%06d.png", frame_dir),
            "-c:v",
            "libx264",
            "-crf",
            "16",
            "-preset",
            "veryslow",
            "-pix_fmt",
            "yuv420p",
            "-progress",
            "pipe:1",
            output_path,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    debug!("ffmpeg process spawned");

    let stdout = process.stdout.unwrap();
    let stdout_reader = BufReader::new(stdout);

    let stderr = process.stderr.unwrap();
    let stderr_reader = BufReader::new(stderr);
    let error_sender = progress_sender.clone();

    let error_thread = thread::spawn(move || -> Result<(), String> {
        for line in stderr_reader.lines() {
            if let Ok(line) = line {
                debug!("stderr line: {}", line);
                if line.contains("warning") {
                    warn!("Detected ffmpeg warning: {}", line);
                } else if line.contains("warning") || line.contains("fatal") {
                    error!("Detected ffmpeg error: {}", line);
                    let message = EncodingMessage::Error(line.clone());
                    let _ = error_sender.send(message);
                    return Err(line);
                }
            }
        }
        Ok(())
    });

    for line in stdout_reader.lines() {
        if let Ok(line) = line {
            if line.starts_with("frame=") {
                let frame_str = line[6..].split_whitespace().next();
                if let Ok(frame) = frame_str.unwrap().parse::<u32>() {
                    let progress = frame as f32 / total_frames as f32;
                    debug!("frames_to_video progress: {}", progress);
                    let message = EncodingMessage::Progress(progress);
                    progress_sender.send(message)?;
                }
            }
        }
    }

    match error_thread.join() {
        Ok(Ok(())) => {
            if progress_sender.send(EncodingMessage::Complete).is_err() {
                warn!("Completion receiver dropped");
            }
        }
        Ok(Err(_)) => {}
        Err(err) => {
            error!("Error thread panicked: {:?}", err);
        }
    }

    Ok(())
}

pub fn frames_to_video_stub(
    frame_dir: &str,
    fps: f64,
    output_path: &str,
    total_frames: u32,
    progress_sender: mpsc::Sender<EncodingMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::{thread, time};

    debug!(
        "Simulating: frame_dir = {}, fps = {}, output_path = {}, total_frames = {}",
        frame_dir, fps, output_path, total_frames
    );

    let duration = time::Duration::from_millis(10);
    let steps = 100; // Total steps to simulate
    let step_progress = 1.0 / steps as f32;

    for step in 0..=steps {
        let progress = step as f32 * step_progress;
        progress_sender.send(EncodingMessage::Progress(progress))?;
        debug!("frames_to_video_stub progress: {}", progress);
        thread::sleep(duration);
    }

    debug!("Simulated video encoding complete");

    if progress_sender.send(EncodingMessage::Complete).is_err() {
        warn!("Completion receiver dropped");
    }

    Ok(())
}
