struct Params {
    // [ax, ay, bx, by]
    ref_points: vec4f,   
    // [points_per_segment, noise_scale, angle_variation, n_lines]
    settings: vec4f,
}

@group(0) @binding(0)
var<uniform> params: Params;

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(0) point_color: vec4f,
}

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOutput {
    let points_per_segment = params.settings.x;
    let noise_scale = params.settings.y;
    let angle_variation = params.settings.z;
    let n_lines = params.settings.w;

    // First figure out which line this vertex belongs to
    let line_idx = floor(f32(v_idx) / points_per_segment);
    
    // Then calculate the point position within that line
    let point_in_line = f32(v_idx % u32(points_per_segment));
    // 0 to 1 within line
    let t = point_in_line / (points_per_segment - 1.0); 
    
    // Calculate vertical position for this line
    let line_y = mix(-0.95, 0.95, line_idx / (n_lines - 1.0)); 
    
    // Base position interpolated between reference points
    let ref_a = vec2f(params.ref_points.x, line_y);
    let ref_b = vec2f(params.ref_points.z, line_y);
    
    let base_pos = mix(ref_a, ref_b, t);

    // Rest of the noise and angle calculations...
    var out: VertexOutput;
    out.pos = vec4f(base_pos, 0.0, 1.0);
    out.point_color = vec4f(0.0, 0.0, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(@location(0) point_color: vec4f) -> @location(0) vec4f {
    return vec4f(0.0, 0.0, 0.0, 1.0);
}

fn rand_pcg(seed: u32) -> f32 {
    var state = seed * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    var result = (word >> 22u) ^ word;
    return f32(result) / 4294967295.0;
}

fn random_normal(seed: u32, mean: f32, stddev: f32) -> f32 {
    let u1 = rand_pcg(seed);
    let u2 = rand_pcg(seed + 1u);
    
    let mag = sqrt(-2.0 * log(u1));
    let z0 = mag * cos(6.28318530718 * u2);
    
    return mean + stddev * z0;
}