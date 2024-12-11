# lattice

A hobbyist project exploring generative art while learning Rust and
[nannou][nannou-link].

Stuff like this:

<img src="images/1000x/displacement_2-tm8s9.png" alt="displacement_2-tm8s9" width="400">

You can see more here on github by looking at the auto generated
[markdown index](index.md).

## Overview

This project aims to port my [p5.js project][p5-link] to Rust for improved
performance. It provides a personal framework around nannou that simplifies
creating multiple sketches by handling common concerns like window creation, GUI
controls, and declarative frame-based animation.

## Planned Features TODO

- [x] Export frame captures and generate MP4 videos for any sketch
- [ ] MIDI synchronization options:
  - [x] Restart frame counter when receiving MIDI start signal
  - [x] Queue frame/video recording when receiving MIDI start signal
  - Send start signal to DAW when starting sketch
- [ ] Receive MIDI enable parameter automation via automation lanes (MIDI CC)
- [x] Common controls system:
  - Shared controls available to all sketches
  - Declarative per-sketch control definitions
  - Framework-agnostic design (currently using egui but another impl could be
    swapped in without impacting sketches I think)
- [x] BPM/musical timing based keyframe animations
  - Support keyframe expressions like `animate([0.0, 1.0, 0.0], 1.5)` to animate
    values over musical beats
- [x] Automatic store/recall of GUI control/parameters
- [x] Audio reactivity. Basic peak, rms, and FFT available to use in sketches

## Status

The project is under active development. Basic animation features are working,
with more functionality planned.

## Requirements

This project requires or optionally needs:

- Rust
- Git LFS for screenshot storage (perhaps this is optional? I'm not too familiar
  with Git LFS but I'm using it for this so you might want to too)
- (optional) [just][just-link] for running commands
- (optional) ffmpeg available on your path for video exports

## Usage

### Creating a new sketch:

1. Copy the [template sketch][template-link] into a new file in sketches folder.
2. Rename at a minimum the `SKETCH_CONFIG.name` field at the top of the file:
   ```rust
   pub const SKETCH_CONFIG: SketchConfig = SketchConfig {
       name: "template", // <-- RENAME THIS!
   ```
3. Add that filename to the [sketches module][module-link]
4. Add a match case for the sketch in [src/main.rs][main-link]:
   ```rust
   "my_awesome_sketch" => run_sketch!(my_awesome_sketch),
   ```
5. Run that sketch via command line by `cargo run --release <name>` or
   `just start <name>` where `name` is what you put in your file's
   `SKETCH_CONFIG.name` field.

[nannou-link]: https://github.com/nannou-org/nannou
[p5-link]: https://github.com/Lokua/p5/tree/main
[just-link]: https://github.com/casey/just
[template-link]: src/sketches/template.rs
[module-link]: src/sketches/mod.res
[main-link]: src/main.rs
