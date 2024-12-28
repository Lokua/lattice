const PI: f32 = 3.14159265359;

struct Params {
    // [ax, ay, bx, by]
    ref_points: vec4f,   
    // [points_per_segment, noise_scale, angle_variation, n_lines]
    settings: vec4f,
    // [point_size, ...unused]
    settings2: vec4f,
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
    let point_size = params.settings2.x;

    // 6u = 6 vertices per triangle = 2 triangles = 1 quad
    let point_index = v_idx / 6u;
    let corner_index = v_idx % 6u;

    // First figure out which line this vertex belongs to
    let line_idx = floor(f32(point_index) / points_per_segment);
    
    // Then calculate the point position within that line
    let point_in_line = f32(point_index % u32(points_per_segment));
    // 0 to 1 within line
    let t = point_in_line / (points_per_segment - 1.0); 
    
    // Calculate vertical position for this line
    let line_y = mix(-0.9, 0.9, line_idx / (n_lines - 1.0)); 
    
    // Base position interpolated between reference points
    let ref_a = vec2f(params.ref_points.x, line_y);
    let ref_b = vec2f(params.ref_points.z, line_y);

    let line_dir = normalize(ref_b - ref_a);
    let perp_dir = vec2f(-line_dir.y, line_dir.x);
    
    let base_pos = mix(ref_a, ref_b, t);

    let base_angle = PI * 0.5;
    let angle = base_angle + random_normal(point_index + 1u, angle_variation);
    let noise = random_normal(point_index, 1.0) * noise_scale;

    let rotated_dir = vec2f(
        perp_dir.x * cos(angle) - perp_dir.y * sin(angle),
        perp_dir.x * sin(angle) + perp_dir.y * cos(angle)
    );
    
    let adjusted_pos = base_pos + rotated_dir * noise;
    let final_pos = adjusted_pos + get_corner_offset(corner_index, point_size);

    var out: VertexOutput;
    out.pos = vec4f(final_pos, 0.0, 1.0);
    out.point_color = vec4f(0.0, 0.0, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(@location(0) point_color: vec4f) -> @location(0) vec4f {
    return point_color;
}

fn get_corner_offset(index: u32, point_size: f32) -> vec2f {
    let s = point_size;
    switch (index) {
        case 0u: { return vec2f(-s, -s); } // bottom-left
        case 1u: { return vec2f(-s,  s); } // top-left
        case 2u: { return vec2f( s,  s); } // top-right
        case 3u: { return vec2f(-s, -s); } // bottom-left
        case 4u: { return vec2f( s,  s); } // top-right
        case 5u: { return vec2f( s, -s); } // bottom-right
        default: { return vec2f(0.0, 0.0); }
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