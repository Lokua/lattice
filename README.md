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
