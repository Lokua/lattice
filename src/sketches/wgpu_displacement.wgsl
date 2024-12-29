struct VertexInput {
    @location(0) position: vec2f,
};

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

struct Params {
    radius: f32,
    strength: f32,
    scaling_power: f32,
    mix: f32,
    r: f32,
    g: f32,
    b: f32,
    ring_strength: f32,
    angular_variation: f32,
    threshold: f32,
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
    let rings = sin(disp_length * params.ring_strength) * 0.5 + 0.5;
    
    // Add angular variation
    let angular_pattern = sin(angle * params.angular_variation) * 0.5 + 0.5;
    
    // Create threshold effects
    let threshold = step(params.threshold, rings * angular_pattern);
    
    // Mix different effects based on params.d
    let pattern = mix(rings * angular_pattern, threshold, params.mix);
    
    // Create black regions where influence is very low
    if (max_influence < 0.01) {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }
    
    return vec4f(
        pattern * params.r, 
        pattern * params.g, 
        pattern * params.b, 
        1.0
    );
}

fn displace(point: vec2f, displacer_pos: vec2f) -> vec2f {
    let distance_from_displacer = distance(displacer_pos, point);
    let proximity = 1.0 - distance_from_displacer / (params.radius * 2.0);
    let distance_factor = max(proximity, 0.0);
    let angle = atan2(point.y - displacer_pos.y, point.x - displacer_pos.x);
    let force = params.strength * pow(distance_factor, params.scaling_power);
    
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