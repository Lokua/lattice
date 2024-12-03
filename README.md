# lattice

A hobbyist project exploring generative art while learning Rust and
[nannou](https://github.com/nannou-org/nannou).

## Overview

This project aims to port my [p5 project](https://github.com/Lokua/p5/tree/main)
to Rust for improved performance. It provides a framework around nannou that
simplifies creating multiple sketches by handling common concerns like window
creation and GUI controls.

## Planned Features

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
- [🚧] BPM/musical timing based keyframe animations
  - Support expressions like `animate([0.0, 1.0, 0.0], 1.5)` to animate values
    over musical beats
  - Basic normalized and normalized ping-pong animations implemented
  - More features in development

## Status

The project is under active development. Basic animation features are working,
with more functionality planned.

## Usage

To create a new sketch:

1. Copy the [template sketch](src/sketches/template.rs) into a new file in
   sketches folder.
2. Rename the `SKETCH_CONFIG` fields at the top of that file accordingly (most
   importantly the `name` field).
3. Add that file to the [sketches module](src/sketches/mod.res)
4. Run that sketch via command line by `cargo run --release <name>` or
   `just start <name>` where `name` is what you put in your file's
   SKETCH_CONFIG.name field (requires that [just](https://github.com/casey/just)
   is installed globally).
