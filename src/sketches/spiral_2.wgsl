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

    // point_size, col_freq, width, unused
    c: vec4f,

    // bg_brightness, time, invert, animate_angle_offset
    d: vec4f,

    // stripe_step, stripe_mix, stripe_amp, stripe_freq
    e: vec4f,

    // animate_bg, circle_radius, circle_phase, wave_amp
    f: vec4f,

    // center_count, center_spread, center_falloff, unused
    g: vec4f,

    // stripe_min, stripe_phase, harmonic_influence, stripe_max
    h: vec4f,
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
    let col_freq = params.c.y;
    let width = params.c.z;
    let harmonic_influence = params.h.z;

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
    
    let circle_pos = get_pos(
        t, 
        line_idx, 
        col_freq * (1.0 + 0.2 * sin(spiral_angle)), 
        width,
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

    let w = params.resolution.x;
    let h = params.resolution.y;
    let aspect = w / h;
    adjusted_pos.x /= aspect;

    let modulation_factor = 1.0 + harmonic_influence * combined_harmonic;
    let dynamic_point_size = point_size * modulation_factor;

    let final_pos = adjusted_pos + 
        get_corner_offset(corner_index, dynamic_point_size);

    var out: VertexOutput;
    out.pos = vec4f(final_pos, 0.0, 1.0);
    
    let alpha = 0.1 * modulation_factor;
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
    let invert = params.d.z;
    let animate_bg = params.f.x;

    let pixel_pos = vec2u(floor(pos.xy));
    var time_seed = 0u;
    if animate_bg == 1.0 { 
        time_seed = u32(time * 1000.0);
    }
    let noise_seed = pixel_pos.x + pixel_pos.y * 1000u + time_seed;
    
    let fine_noise = rand_pcg(noise_seed);
    let very_fine_noise = rand_pcg(noise_seed * 31u + 17u);
    let combined_noise = mix(fine_noise, very_fine_noise, 0.5);
    
    let brightness = combined_noise * bg_brightness;
    let background_color = vec4f(vec3f(brightness), 1.0);

    let color = select(background_color, point_color, point_color.a > 0.0);
    if invert == 1.0 {
        return vec4f(1.0 - color.r, 1.0 - color.g, 1.0 - color.b, color.a);
    }
    return color;
}

fn get_pos(
    t: f32, 
    line_idx: f32, 
    col_freq: f32,
    width: f32,
    spiral_factor: f32,
) -> vec2f {
    let distortion = params.c.w;
    let time = params.d.y;
    let animate_angle_offset = params.d.w;
    let n_lines = params.b.w;
    let wave_amp = params.f.w;
    let center_count = params.g.x;  
    let center_spread = params.g.y; 
    let center_falloff = params.g.z;
    let circle_radius = params.f.y;  // Controls actual circle size
    let circle_phase = params.f.z; 
    
    // Base grid position
    let x = (t * 2.0 - 1.0) * width;
    let y = ((line_idx / (n_lines - 1.0)) * 2.0 - 1.0) * width;
    
    // Create frequency-based displacement
    let freq = max(col_freq, 0.1);
    let displacement = sin(x * freq) * width * wave_amp;
    
    var pos = vec2f(x, y + displacement);
    
    var total_distortion = vec2f(0.0);
    
    // Create distortion from multiple centers
    for (var i = 0.0; i < center_count; i += 1.0) {
        // Calculate center position in a circle
        let center_angle = (i / center_count) * TAU + circle_phase;
        let center_pos = vec2f(
            cos(center_angle) * center_spread * width,
            sin(center_angle) * center_spread * width
        );
        
        // Calculate distance to this center
        let delta = pos - center_pos;
        let dist = length(delta);
        
        // Use circle_radius to define the actual size of each distortion circle
        let circle_size = width * circle_radius;
        
        // Create a more defined circle edge using smoothstep
        let circle_edge = smoothstep(circle_size, circle_size * 0.8, dist);
        let strength = circle_edge * distortion * width * 0.2;
        
        // Add radial distortion
        total_distortion += normalize(delta) * strength;
    }
    
    // Apply distortion with size preservation
    let distortion_length = length(total_distortion);
    let max_distortion = width * 0.3; // Maximum allowed distortion
    if distortion_length > 0.0 {
        total_distortion = normalize(total_distortion) * 
            min(distortion_length, max_distortion);
    }
    
    pos += total_distortion;
    
    let angle = atan2(pos.y, pos.x);
    let final_r = length(pos);
    
    var r_modulated = apply_stripe_modulation(final_r, angle);
    
    pos *= r_modulated / max(final_r, 0.0001);
    
    return pos;
}

fn apply_stripe_modulation(radius: f32, pos_angle: f32) -> f32 {
    let stripe_step = params.e.x;
    let stripe_mix = params.e.y;
    let stripe_amp = params.e.z;
    let stripe_freq = params.e.w;
    let stripe_phase = params.h.y;
    let stripe_min = params.h.x;
    let stripe_max = params.h.w;
    let normalized_phase = stripe_phase * stripe_freq;
    let stripe_input = sin(stripe_freq * pos_angle + normalized_phase);
    let stripe1 = step(stripe_step, stripe_input); 
    let stripe2 = smoothstep(stripe_min, stripe_max, stripe_input); 
    let stripe = mix(stripe1, stripe2, stripe_mix);
    let modulation = 1.0 + stripe_amp * (2.0 * stripe - 1.0);
    return radius * modulation;
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
