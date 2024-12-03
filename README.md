# lattice

A hobbyist project exploring generative art while learning Rust and
[nannou][nannou-link].

## Overview

This project aims to port my [p5 project][p5-link] to Rust for improved
performance. It provides a framework around nannou that simplifies creating
multiple sketches by handling common concerns like window creation and GUI
controls.

## Planned Features TODO

- [ ] Export frame captures and generate MP4 videos for any sketch
- [ ] MIDI synchronization options:
  - Restart frame counter when receiving DAW start signal
  - Send start signal to DAW when starting sketch
- [ ] Receive MIDI data from DAW to enable parameter automation via automation
      lanes
- [x] Common controls system:
  - Shared controls available to all sketches
  - Declarative per-sketch control definitions
  - Framework-agnostic design (currently using egui but another impl could be
    swapped in without impacting sketches I think)
- [ ] BPM/musical timing based keyframe animations 🚧
  - Support expressions like `animate([0.0, 1.0, 0.0], 1.5)` to animate values
    over musical beats
  - Basic normalized and normalized ping-pong animations implemented
  - More features in development
- [ ] Automatic store/recall of GUI control/parameters

## Status

The project is under active development. Basic animation features are working,
with more functionality planned.

## Requirements

This project requires or optionally needs:

- Rust
- Git LFS for screenshot storage (perhaps this is optional? I'm not too familiar
  with Git LFS but I'm using it for this so you might want to too)
- (optional) [just][just-link] for running commands

## Usage

To create a new sketch:

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
