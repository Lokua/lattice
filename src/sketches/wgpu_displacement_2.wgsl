struct VertexInput {
    @location(0) position: vec2f,
};

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

struct Params {
    // center, top-right, bottom-right, bottom-left, top-left
    // [radius, strength, scale, unused]
    d_0: vec4f,
    d_1: vec4f,
    d_2: vec4f,
    d_3: vec4f,
    d_4: vec4f,

    // displacer "instance" params
    radius: f32,
    strength: f32,
    scaling_power: f32,

    // "global" params
    r: f32,
    g: f32,
    b: f32,
    offset: f32,
    ring_strength: f32,
    angular_variation: f32,
    threshold: f32,
    mix: f32,

    _pad: f32,
}

@group(0) @binding(0)
var<uniform> params: Params;

@vertex
fn vs_main(vert: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4f(vert.position, 0.0, 1.0);
    out.uv = vert.position * 0.5 + 0.5;
    return out;
}

@fragment
fn fs_main(@location(0) uv: vec2f) -> @location(0) vec4f {
    var total_displacement = vec2f(0.0);
    var max_influence = 0.0;

    for (var i = 0u; i < 5u; i++) {
        let pos = get_displacer_position(i);
        let displacer_params = get_displacer_params(i);
        let displacement = displace(uv, pos, displacer_params);
        total_displacement += displacement;
        max_influence = max(max_influence, length(displacement));
    }
    
    // Create patterns based on influence strength
    let disp_length = length(total_displacement);
    let angle = atan2(total_displacement.y, total_displacement.x);

    let rings = sin(disp_length * params.ring_strength) * 
        sin(disp_length * params.ring_strength * 0.5) * 0.5 + 0.5;
    
    // Add angular variation
    let angular_pattern = sin(angle * params.angular_variation) * 0.5 + 0.5;
    
    // Create threshold effects
    let threshold = step(params.threshold, rings * angular_pattern);
    
    // Mix different effects based on params.d
    let pattern = mix(rings * angular_pattern, threshold, params.mix);
    
    // Create black regions where influence is very low
    if max_influence < 0.01 {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }

    // Make colors shift based on angle
    let hue_shift = (sin(angle * 2.0) + sin(angle * 3.0)) * 0.25 + 0.5;

    return vec4f(
        pattern * params.r * hue_shift,
        pattern * params.g * (1.0 - hue_shift),
        pattern * params.b,
        1.0
    );
}

fn displace(
    point: vec2f, 
    displacer_pos: vec2f, 
    displacer_params: vec4f
) -> vec2f {
    let radius = displacer_params.x;
    let strength = displacer_params.y;
    let scaling_power = displacer_params.z;
    
    let distance_from_displacer = distance(displacer_pos, point);
    let proximity = 1.0 - distance_from_displacer / (radius * 2.0);
    let distance_factor = max(proximity, 0.0);
    let angle = atan2(point.y - displacer_pos.y, point.x - displacer_pos.x);
    let force = strength * pow(distance_factor, scaling_power);

    return vec2f(
        cos(angle) * force,
        sin(angle) * force
    );
}

fn get_displacer_position(index: u32) -> vec2f {
    switch(index) {
        // Center
        case 0u: { return vec2f(0.5); }
        // Top right
        case 1u: { return vec2f(1.0 - params.d_1.w, params.d_1.w); }
        // Bottom right 
        case 2u: { return vec2f(1.0 - params.d_2.w); }
        // Bottom left 
        case 3u: { return vec2f(params.d_3.w); }
        // Top left 
        case 4u: { return vec2f(params.d_4.w, 1.0 - params.d_4.w); }
        // Default case required
        default: { return vec2f(0.0, 0.0); }
    }
}

fn get_displacer_params(index: u32) -> vec4f {
    switch(index) {
        // Center
        case 0u: { return params.d_0; }
        // Top right
        case 1u: { return params.d_1; }
        // Bottom right 
        case 2u: { return params.d_2; }
        // Bottom left 
        case 3u: { return params.d_3; }
        // Top left 
        case 4u: { return params.d_4; }
        // Default case required
        default: { return vec4f(0.0); }
    }
}