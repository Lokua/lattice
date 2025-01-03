pub mod displacement_1;
pub mod displacement_1a;
pub mod displacement_1b;
pub mod displacement_1b_animated;
pub mod displacement_2;
pub mod displacement_2a;
pub mod drop;
pub mod drop_walk;
pub mod genuary_1;
pub mod sand_lines;
pub mod sand_lines_wgpu;
pub mod template;
pub mod template_wgpu;
pub mod wgpu_displacement;
pub mod wgpu_displacement_2;

pub mod scratch;
pub use self::scratch::animation_test;
pub use self::scratch::audio_test;
pub use self::scratch::bos;
pub use self::scratch::chromatic_aberration;
pub use self::scratch::lines;
pub use self::scratch::midi_test;
pub use self::scratch::noise;
pub use self::scratch::perlin_loop;
pub use self::scratch::responsive_test;
pub use self::scratch::sand_line;
pub use self::scratch::vertical;
pub use self::scratch::vertical_2;
pub use self::scratch::wgpu_compute_test;
pub use self::scratch::wgpu_test;
pub use self::scratch::z_sim;
