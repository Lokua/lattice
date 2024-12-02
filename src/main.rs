use crate::framework::frame_controller;
use crate::framework::logger::init_logger;
use log::{info, warn};
use std::env;

pub mod framework;
mod sketches;

fn main() {
    init_logger();

    let args: Vec<String> = env::args().collect();
    let sketch_name = args.get(1).map(|s| s.as_str()).unwrap_or("template");

    match sketch_name {
        "template" => {
            info!("Loading {}", sketches::template::METADATA.display_name);
            frame_controller::init_controller(sketches::template::METADATA.fps);

            nannou::app(sketches::template::model)
                .update(|app, model, update| {
                    frame_controller::wrapped_update(
                        app,
                        model,
                        update,
                        sketches::template::update,
                    )
                })
                .view(|app, model, frame| {
                    frame_controller::wrapped_view(
                        app,
                        model,
                        frame,
                        sketches::template::view,
                    )
                })
                .run();
        }
        "displacement_1" => {
            info!(
                "Loading {}",
                sketches::displacement_1::METADATA.display_name
            );
            frame_controller::init_controller(
                sketches::displacement_1::METADATA.fps,
            );

            nannou::app(sketches::displacement_1::model)
                .update(|app, model, update| {
                    frame_controller::wrapped_update(
                        app,
                        model,
                        update,
                        sketches::displacement_1::update,
                    )
                })
                .view(|app, model, frame| {
                    frame_controller::wrapped_view(
                        app,
                        model,
                        frame,
                        sketches::displacement_1::view,
                    )
                })
                .run();
        }
        _ => {
            warn!("Sketch not found, running template");
            info!("Loading {}", sketches::template::METADATA.display_name);
            frame_controller::init_controller(sketches::template::METADATA.fps);

            nannou::app(sketches::template::model)
                .update(|app, model, update| {
                    frame_controller::wrapped_update(
                        app,
                        model,
                        update,
                        sketches::template::update,
                    )
                })
                .view(|app, model, frame| {
                    frame_controller::wrapped_view(
                        app,
                        model,
                        frame,
                        sketches::template::view,
                    )
                })
                .run();
        }
    }
}
