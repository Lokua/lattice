const PI: f32 = 3.14159265359;
const TAU: f32 = 6.283185307179586;
const PHI: f32 = 1.61803398875;

const FULLSCREEN_TRIANGLE_VERTS = array<vec2f, 3>(
    vec2f(-1.0, -3.0),
    vec2f( 3.0,  1.0),
    vec2f(-1.0,  1.0)
);

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(0) point_color: vec4f,
    @location(1) uv: vec2f,
}

struct Params {
    // w, h, ..unused
    resolution: vec4f,

    // ax, ay, bx, by
    a: vec4f,

    // points_per_segment, noise_scale, angle_variation, n_lines
    b: vec4f,

    // point_size, circle_r_min, circle_r_max, unused
    c: vec4f,

    // bg_brightness, time, ..unused
    d: vec4f,
}

@group(0) @binding(0)
var<uniform> params: Params;

@vertex
fn vs_main(@builtin(vertex_index) vidx: u32) -> VertexOutput {
        // Use first 3 vertices for background
    if (vidx < 3u) {
        // Full-screen triangle vertices
        var pos = FULLSCREEN_TRIANGLE_VERTS;
        var out: VertexOutput;
        out.pos = vec4f(pos[vidx], 0.0, 1.0);
        // Use uv for noise sampling if needed
        out.uv = (pos[vidx] + 1.0) * 0.5;
        // Background doesn’t need a specific point_color
        out.point_color = vec4f(0.0);
        return out;
    }

    // Adjust index for spiral vertices: subtract background vertex count
    let vert_index = vidx - 3u;

    let points_per_segment = params.b.x;
    let noise_scale = params.b.y;
    let angle_variation = params.b.z;
    let n_lines = params.b.w;
    let point_size = params.c.x;
    let circle_r_min = params.c.y;
    let circle_r_max = params.c.z;

    let total_points_per_pass = u32(n_lines * points_per_segment);
    let point_index = (vert_index / 6u) % total_points_per_pass;
    let corner_index = vert_index % 6u;
    let line_idx = floor(f32(point_index) / points_per_segment);
    let point_in_line = f32(point_index) % points_per_segment;
    let t = point_in_line / (points_per_segment - 1.0);

    // Distribute lines evenly in vertical space
    let step = 1.8 / (n_lines - 1.0);
    let offset = (n_lines - 1.0) * 0.5;
    let y_pos = (line_idx - offset) * step;

    let base_freq = TAU;
    // Use line_idx directly for phase to ensure unique offset per line
    let phase_offset = line_idx * 0.1;
    
    let harmonic1 = sin(t * base_freq + phase_offset);
    let harmonic2 = sin(t * base_freq + phase_offset * 2.0) * 0.5;
    let harmonic3 = sin(t * base_freq + phase_offset * 3.0) * 0.3;
    
    let combined_harmonic = harmonic1 + harmonic2 + harmonic3;

    let noise_seed = point_index + 1u;
    let noise = random_normal(noise_seed, 1.0) * 
        noise_scale * 
        (1.0 + abs(combined_harmonic));

    let spiral_factor = line_idx / n_lines;
    let spiral_angle = t * TAU + spiral_factor * TAU;
    
    let circle_pos = get_circle_pos(
        t, 
        line_idx, 
        n_lines, 
        circle_r_min * (1.0 + 0.2 * sin(spiral_angle)), 
        circle_r_max,
        spiral_factor
    );

    let angle = random_normal(point_index, angle_variation) + 
        spiral_angle * 0.5 + 
        combined_harmonic * 0.3;

    let ref_a = vec2f(params.a.x, y_pos);
    let ref_b = vec2f(params.a.z, y_pos);
    let line_dir = normalize(ref_b - ref_a);
    let perp_dir = vec2f(-line_dir.y, line_dir.x);
    
    let rotated_dir = vec2f(
        perp_dir.x * cos(angle) - perp_dir.y * sin(angle),
        perp_dir.x * sin(angle) + perp_dir.y * cos(angle)
    );

    var adjusted_pos = circle_pos + 
        rotated_dir * 
        noise * 
        (1.0 + 0.3 * combined_harmonic);

    let dynamic_point_size = point_size * (1.0 + 0.2 * combined_harmonic);

    let w = params.resolution.x;
    let h = params.resolution.y;
    let aspect = w / h;
    adjusted_pos.x /= aspect;

    let final_pos = adjusted_pos + 
        get_corner_offset(corner_index, dynamic_point_size);

    var out: VertexOutput;
    out.pos = vec4f(final_pos, 0.0, 1.0);
    
    let alpha = 0.1 * (1.0 + 0.2 * combined_harmonic);
    out.point_color = vec4f(vec3f(0.0), alpha);
    out.uv = (final_pos.xy + 1.0) * 0.5;
    return out;
}

@fragment
fn fs_main(
    @builtin(position) pos: vec4f,
    @location(0) point_color: vec4f,
    @location(1) uv: vec2f,
) -> @location(0) vec4f {
    let bg_brightness = params.d.x;
    let time = params.d.y;

    let pixel_pos = vec2u(floor(pos.xy));
    let time_seed = u32(time * 1000.0); 
    // let time_seed = 0u; 
    let noise_seed = pixel_pos.x + pixel_pos.y * 1000u + time_seed;
    
    let fine_noise = rand_pcg(noise_seed);
    let very_fine_noise = rand_pcg(noise_seed * 31u + 17u);
    let combined_noise = mix(fine_noise, very_fine_noise, 0.5);
    
    // Use noise value to modulate brightness
    let brightness = combined_noise * bg_brightness;
    let background_color = vec4f(vec3f(brightness), 1.0);

    // If point_color.a > 0.0, use point_color; otherwise use background
    return select(background_color, point_color, point_color.a > 0.0);
}

fn get_circle_pos(
    t: f32, 
    line_idx: f32, 
    n_lines: f32, 
    min_r: f32, 
    max_r: f32,
    spiral_factor: f32,
) -> vec2f {
    let offset_mult = params.c.w;
    let radius_factor = line_idx / n_lines;
    let actual_min = min(min_r, max_r);
    let actual_max = max(min_r, max_r);
    
    // Keep the radius interpolation direction-aware
    let invert_factor = select(radius_factor, 1.0 - radius_factor, min_r > max_r);
    let radius = mix(actual_min, actual_max, invert_factor);
    
    // Maintain spiral direction but adjust the phase
    let direction = select(1.0, -1.0, min_r > max_r);
    let angle_offset = direction * pow(radius_factor, PHI) * TAU * offset_mult;
    let pos_angle = t * TAU + angle_offset;
    
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
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    var result = (word >> 22u) ^ word;
    return f32(result) / 4294967295.0;
}

fn random_normal(seed: u32, std_dev: f32) -> f32 {
    let u1 = rand_pcg(seed);
    let u2 = rand_pcg(seed + 1u);
    let mag = sqrt(-2.0 * log(u1));
    let z0 = mag * cos(2.0 * PI * u2);
    return std_dev * z0;
}