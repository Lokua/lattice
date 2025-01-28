pub mod blob;
pub mod displacement_1;
pub mod displacement_1a;
pub mod displacement_2;
pub mod displacement_2a;
pub mod drop;
pub mod drop_walk;
pub mod floor_supervisor;
pub mod flow_field_basic;
pub mod heat_mask;
pub mod interference;
pub mod sand_lines;
pub mod sierpinski_triangle;
pub mod spiral;
pub mod spiral_2;
pub mod template;
pub mod template_wgpu;
pub mod wave_fract;
pub mod wgpu_displacement;
pub mod wgpu_displacement_2;

pub mod genuary_2025;
pub use self::genuary_2025::g25_10_11_12;
pub use self::genuary_2025::g25_13_triangle;
pub use self::genuary_2025::g25_14_blank_and_white;
pub use self::genuary_2025::g25_18_wind;
pub use self::genuary_2025::g25_19_op_art;
pub use self::genuary_2025::g25_1_horiz_vert;
pub use self::genuary_2025::g25_22_gradients_only;
pub use self::genuary_2025::g25_26_symmetry;
pub use self::genuary_2025::g25_2_layers;
pub use self::genuary_2025::g25_5_isometric;

pub mod scratch;
pub use self::scratch::animation_script_test;
pub use self::scratch::animation_test;
pub use self::scratch::audio_test;
pub use self::scratch::bos;
pub use self::scratch::chromatic_aberration;
pub use self::scratch::cv_test;
pub use self::scratch::lin_alg;
pub use self::scratch::lines;
pub use self::scratch::midi_test;
pub use self::scratch::noise;
pub use self::scratch::osc_test;
pub use self::scratch::osc_transport_test;
pub use self::scratch::perlin_loop;
pub use self::scratch::responsive_test;
pub use self::scratch::sand_line;
pub use self::scratch::shader_experiments;
pub use self::scratch::vertical;
pub use self::scratch::vertical_2;
pub use self::scratch::wgpu_compute_test;
pub use self::scratch::wgpu_test;
pub use self::scratch::z_sim;
