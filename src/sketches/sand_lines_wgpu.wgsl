const PI: f32 = 3.14159265359;

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(0) point_color: vec4f,
}

struct Params {
    // w, h, ..unused
    resolution: vec4f,

    // ax, ay, bx, by
    ref_points: vec4f,

    // points_per_segment, noise_scale, angle_variation, n_lines
    settings: vec4f,

    // point_size, circle_r_min, circle_r_max
    settings2: vec4f,
}

@group(0) @binding(0)
var<uniform> params: Params;

@vertex
fn vs_main(@builtin(vertex_index) vert_index: u32) -> VertexOutput {
    let points_per_segment = params.settings.x;
    let noise_scale = params.settings.y;
    let angle_variation = params.settings.z;
    let n_lines = params.settings.w;
    let point_size = params.settings2.x;
    let circle_r_min = params.settings2.y;
    let circle_r_max = params.settings2.z;

    let total_points_per_pass = u32(n_lines * points_per_segment);
    let point_index  = (vert_index / 6u) % total_points_per_pass;
    let corner_index = vert_index % 6u;
    let line_idx = floor(f32(point_index) / points_per_segment);
    let point_in_line = f32(point_index) % points_per_segment;
    let t = point_in_line / (points_per_segment - 1.0);

    let step   = 1.8 / (n_lines - 1.0);
    let offset = (n_lines - 1.0) * 0.5;
    let y_pos  = (line_idx - offset) * step;

    let angle_seed = point_index;
    let angle = random_normal(point_index, angle_variation);

    let noise_seed = angle_seed + 1u;
    let noise = random_normal(noise_seed, 1.0) * noise_scale;

    let ref_a = vec2f(params.ref_points.x, y_pos);
    let ref_b = vec2f(params.ref_points.z, y_pos);

    let line_dir = normalize(ref_b - ref_a);
    let perp_dir = vec2f(-line_dir.y, line_dir.x);
    let rotated_dir = vec2f(
        perp_dir.x * cos(angle) - perp_dir.y * sin(angle),
        perp_dir.x * sin(angle) + perp_dir.y * cos(angle)
    );

    let circle_pos = get_circle_pos(
        t, 
        line_idx, 
        n_lines, 
        circle_r_min, 
        circle_r_max
    );

    var adjusted_pos = circle_pos + rotated_dir * noise;

    let w = params.resolution.x;
    let h = params.resolution.y;
    let aspect = w / h;
    adjusted_pos.x /= aspect;

    let final_pos = adjusted_pos + get_corner_offset(corner_index, point_size);

    var out: VertexOutput;
    out.pos = vec4f(final_pos, 0.0, 1.0);
    out.point_color = vec4f(vec3f(0.0), 0.1);
    return out;
}

@fragment
fn fs_main(@location(0) point_color: vec4f) -> @location(0) vec4f {
    return point_color;
}

fn get_circle_pos(
    t: f32, 
    line_idx: f32, 
    n_lines: f32, 
    min_r: f32, 
    max_r: f32
) -> vec2f {
    let radius = mix(min_r, max_r, line_idx / n_lines);
    let pos_angle = t * PI * 2.0;
    return vec2f(
        cos(pos_angle) * radius,
        sin(pos_angle) * radius
    );
}

fn get_corner_offset(index: u32, point_size: f32) -> vec2f {
    let s = point_size;
    switch (index) {
        case 0u: { return vec2f(-s, -s); }
        case 1u: { return vec2f(-s,  s); }
        case 2u: { return vec2f( s,  s); }
        case 3u: { return vec2f(-s, -s); }
        case 4u: { return vec2f( s,  s); }
        case 5u: { return vec2f( s, -s); }
        default: { return vec2f(0.0); }
    }
}

fn rand_pcg(seed: u32) -> f32 {
    var state = seed * 747796405u + 2891336453u;
    var word  = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    var result = (word >> 22u) ^ word;
    return f32(result) / 4294967295.0;
}

fn random_normal(seed: u32, std_dev: f32) -> f32 {
    let u1 = rand_pcg(seed);
    let u2 = rand_pcg(seed + 1u);
    let mag = sqrt(-2.0 * log(u1));
    let z0  = mag * cos(2.0 * PI * u2);
    return std_dev * z0;
}