use crate::framework::frame_controller;
use crate::framework::logger::init_logger;
use log::{info, warn};
use std::env;

pub mod framework;
mod sketches;

macro_rules! run_sketch {
    ($sketch:ident) => {{
        info!("Loading {}", sketches::$sketch::METADATA.display_name);
        frame_controller::init_controller(sketches::$sketch::METADATA.fps);

        nannou::app(sketches::$sketch::model)
            .update(move |app, model, update| {
                frame_controller::wrapped_update(
                    app,
                    model,
                    update,
                    sketches::$sketch::update,
                )
            })
            .view(move |app, model, frame| {
                frame_controller::wrapped_view(
                    app,
                    model,
                    frame,
                    sketches::$sketch::view,
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
        "template" => run_sketch!(template),
        "displacement_1" => run_sketch!(displacement_1),
        _ => {
            warn!("Sketch not found, running template");
            run_sketch!(template)
        }
    }
}
