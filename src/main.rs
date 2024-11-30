pub mod framework;
mod sketches;

fn main() {
    use sketches::displacement_1;

    nannou::app(displacement_1::model)
        .update(displacement_1::update)
        .view(displacement_1::view)
        .run();
}
