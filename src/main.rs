use crate::framework::frame_controller;
use crate::framework::logger::init_logger;
use log::info;

pub mod framework;
mod sketches;

fn main() {
    init_logger();

    use sketches::displacement_1;
    info!("Loading {}", displacement_1::METADATA.display_name);

    frame_controller::init_controller(displacement_1::METADATA.fps);

    nannou::app(displacement_1::model)
        .update(|app, model, update| {
            frame_controller::wrapped_update(
                app,
                model,
                update,
                displacement_1::update,
            );
        })
        .view(|app, model, frame| {
            frame_controller::wrapped_view(
                app,
                model,
                frame,
                displacement_1::view,
            );
        })
        .run();
}
