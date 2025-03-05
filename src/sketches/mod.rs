pub mod blob;
pub mod breakpoints_2;
pub mod brutalism;
pub mod displacement_2a;
pub mod drop;
pub mod drop_walk;
pub mod floor_supervisor;
pub mod flow_field_basic;
pub mod heat_mask;
pub mod interference;
pub mod kalos;
pub mod kalos_2;
pub mod sand_lines;
pub mod sierpinski_triangle;
pub mod spiral;
pub mod spiral_lines;
pub mod wave_fract;

pub mod dev;
pub use self::dev::animation_dev;
pub use self::dev::audio_controls_dev;
pub use self::dev::audio_dev;
pub use self::dev::control_script_dev;
pub use self::dev::cv_dev;
pub use self::dev::effects_wavefolder_dev;
pub use self::dev::midi_dev;
pub use self::dev::osc_dev;
pub use self::dev::osc_transport_dev;
pub use self::dev::responsive_dev;
pub use self::dev::shader_to_texture_dev;
pub use self::dev::wgpu_compute_dev;

pub mod genuary_2025;
pub use self::genuary_2025::g25_10_11_12;
pub use self::genuary_2025::g25_13_triangle;
pub use self::genuary_2025::g25_14_black_and_white;
pub use self::genuary_2025::g25_18_wind;
pub use self::genuary_2025::g25_19_op_art;
pub use self::genuary_2025::g25_1_horiz_vert;
pub use self::genuary_2025::g25_20_23_brutal_arch;
pub use self::genuary_2025::g25_22_gradients_only;
pub use self::genuary_2025::g25_26_symmetry;
pub use self::genuary_2025::g25_2_layers;
pub use self::genuary_2025::g25_5_isometric;

pub mod scratch;
pub use self::scratch::bos;
pub use self::scratch::chromatic_aberration;
pub use self::scratch::displacement_1;
pub use self::scratch::displacement_1a;
pub use self::scratch::displacement_2;
pub use self::scratch::lin_alg;
pub use self::scratch::lines;
pub use self::scratch::noise;
pub use self::scratch::perlin_loop;
pub use self::scratch::sand_line;
pub use self::scratch::shader_experiments;
pub use self::scratch::vertical;
pub use self::scratch::vertical_2;
pub use self::scratch::z_sim;

pub mod shared;

pub mod templates;
pub use self::templates::basic_cube_shader_template;
pub use self::templates::fullscreen_shader_template;
pub use self::templates::template;
