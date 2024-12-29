struct VertexInput {
    @location(0) position: vec2f,
};

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

struct Params {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
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
        let displacement = displace(uv, pos);
        total_displacement += displacement;
        max_influence = max(max_influence, length(displacement));
    }
    
    // Create patterns based on influence strength
    let disp_length = length(total_displacement);
    let angle = atan2(total_displacement.y, total_displacement.x);
    
    // Create bands/rings based on displacement
    let rings = sin(disp_length * 20.0) * 0.5 + 0.5;
    
    // Add angular variation
    let angular_pattern = sin(angle * 4.0) * 0.5 + 0.5;
    
    // Create threshold effects
    let threshold = step(0.7, rings * angular_pattern);
    
    // Mix different effects based on params.d
    let pattern = mix(rings * angular_pattern, threshold, params.d);
    
    // Create black regions where influence is very low
    if (max_influence < 0.01) {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }
    
    return vec4f(0.0, 0.0, pattern, 1.0);
}

fn displace(point: vec2f, displacer_pos: vec2f) -> vec2f {
    let radius = params.a;
    let strength = params.b * 2.0;
    let scaling_power = params.c * 4.0;
    
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
        case 1u: { return vec2f(0.8, 0.2); }
        // Top left 
        case 2u: { return vec2f(0.2, 0.8); }
        // Bottom right 
        case 3u: { return vec2f(0.8); }
        // Bottom left 
        case 4u: { return vec2f(0.2); }
        // Default case required
        default: { return vec2f(0.0, 0.0); }
    }
}