pub mod displacement_1;
pub mod displacement_1a;
pub mod displacement_2;
pub mod displacement_2a;
pub mod drop;
pub mod drop_walk;
pub mod floor_supervisor;
pub mod flow_field;
pub mod flow_field_basic;
pub mod genuary_1;
pub mod genuary_14;
pub mod genuary_2;
pub mod genuary_5;
pub mod heat_mask;
pub mod interference;
pub mod sand_lines;
pub mod sierpinski_triangle;
pub mod sierpinski_triangle_auto;
pub mod spiral;
pub mod spiral_2;
pub mod spiral_auto;
pub mod template;
pub mod template_wgpu;
pub mod wave_fract;
pub mod wgpu_displacement;
pub mod wgpu_displacement_2;

pub mod scratch;
pub use self::scratch::animation_script_test;
pub use self::scratch::animation_test;
pub use self::scratch::audio_test;
pub use self::scratch::bos;
pub use self::scratch::chromatic_aberration;
pub use self::scratch::lin_alg;
pub use self::scratch::lines;
pub use self::scratch::midi_test;
pub use self::scratch::noise;
pub use self::scratch::osc_test;
pub use self::scratch::perlin_loop;
pub use self::scratch::responsive_test;
pub use self::scratch::sand_line;
pub use self::scratch::shader_experiments;
pub use self::scratch::vertical;
pub use self::scratch::vertical_2;
pub use self::scratch::wgpu_compute_test;
pub use self::scratch::wgpu_test;
pub use self::scratch::z_sim;
