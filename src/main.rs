use crate::framework::frame_controller;

pub mod framework;
mod sketches;

fn main() {
    use sketches::displacement_1;
    println!("Loading {}", displacement_1::METADATA.name);

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
