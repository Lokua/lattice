use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;

pub fn frames_to_video(
    frame_dir: &str,
    fps: f64,
    output_path: &str,
    total_frames: u32,
    progress_sender: mpsc::Sender<f32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let process = Command::new("ffmpeg")
        .args([
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
        .stderr(Stdio::null())
        .spawn()?;

    let stdout = process.stdout.unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.starts_with("frame=") {
                let frame_str = line[6..].split_whitespace().next();
                if let Ok(frame) = frame_str.unwrap().parse::<u32>() {
                    let progress = frame as f32 / total_frames as f32;
                    progress_sender.send(progress)?;
                }
            }
        }
    }

    Ok(())
}
