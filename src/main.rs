use nannou::LoopMode;

pub mod framework;
mod sketches;

fn main() {
    use sketches::displacement_1;
    println!("Loading {}", displacement_1::METADATA.name);
    nannou::app(displacement_1::model)
        .loop_mode(LoopMode::rate_fps(displacement_1::METADATA.fps))
        .update(displacement_1::update)
        .view(displacement_1::view)
        .run();
}
