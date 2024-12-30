struct VertexInput {
    @location(0) position: vec2f,
};

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) pos: vec2f,
};

struct Params {
    resolution: vec4f,
    radius: f32,
    strength: f32,
    scaling_power: f32,
    r: f32,
    g: f32,
    b: f32,
    offset: f32,
    ring_strength: f32,
    angular_variation: f32,
    threshold: f32,
    mix: f32,
    alg: f32,
    j: f32,
    k: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0)
var<uniform> params: Params;

@vertex
fn vs_main(vert: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4f(vert.position, 0.0, 1.0);
    out.pos = vert.position;
    return out;
}

@fragment
fn fs_main(@location(0) position: vec2f) -> @location(0) vec4f {
    let aspect = params.resolution.x / params.resolution.y;
    var pos = position;
    pos.x *= aspect;

    var total_displacement = vec2f(0.0);
    var max_influence = 0.0;

    for (var i = 0u; i < 5u; i++) {
        let displacer_pos = get_displacer_position(i);
        let displacement = displace(pos, displacer_pos);
        total_displacement += displacement;
        max_influence = max(max_influence, length(displacement));
    }
    
    // Create patterns based on influence strength
    let disp_length = length(total_displacement);
    let angle = atan2(total_displacement.y, total_displacement.x);
    
    // Create bands/rings based on displacement
    let rings = sin(disp_length * params.ring_strength);
    
    // Add angular variation
    let angular_pattern = sin(angle * params.angular_variation);
    
    // Create threshold effects
    let threshold = step(params.threshold, rings * angular_pattern);
    
    // Mix different effects based on params.d
    let pattern = mix(rings * angular_pattern, threshold, params.mix);
    
    // Create black regions where influence is very low
    if max_influence < 0.01 {
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
    var d = 0.0;
    if params.alg == 0.0 {
        d = distance(displacer_pos, point);
    } else if params.alg == 1.0 {
        d = concentric_waves(displacer_pos, point, params.j);
    } else if params.alg == 2.0 {
        d = wave_interference(displacer_pos, point, params.j, params.k);
    }

    let proximity = 1.0 - d / (params.radius * 2.0);
    let distance_factor = max(proximity, 0.0);
    let angle = atan2(point.y - displacer_pos.y, point.x - displacer_pos.x);
    let force = params.strength * pow(distance_factor, params.scaling_power);

    return vec2f(
        cos(angle) * force,
        sin(angle) * force
    );
}

fn concentric_waves(p1: vec2f, p2: vec2f, frequency: f32) -> f32 {
    let distance = length(p2 - p1);
    return abs(sin(distance * frequency)) * exp(-distance * 0.01);
}

fn wave_interference(p1: vec2f, p2: vec2f, frequency: f32, amount: f32) -> f32 {
    let source2 = p1 + vec2f(50.0, 50.0);
    let d1 = distance(p1, p2);
    let d2 = distance(source2, p2);
    return sin(d1 * frequency) + sin(d2 * frequency) * (params.k * 10.0);
}

fn get_displacer_position(index: u32) -> vec2f {
    let max = 1.0 - params.offset;
    let min = -1.0 + params.offset;

    switch(index) {
        // Center
        case 0u: { return vec2f(0.0); }
        // Top right
        case 1u: { return vec2f(max, min); }
        // Top left 
        case 2u: { return vec2f(min, max); }
        // Bottom right 
        case 3u: { return vec2f(max); }
        // Bottom left 
        case 4u: { return vec2f(min); }
        // Default case required
        default: { return vec2f(0.0, 0.0); }
    }
}