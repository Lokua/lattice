const PI: f32 = 3.14159265359;
const TAU: f32 = 6.283185307179586;

struct VertexInput {
    @location(0) position: vec2f,
};

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) pos: vec2f,
};

struct Params {
    resolution: vec4f,
    
    // wave1_frequency, wave1_angle, wave2_frequency, wave2_angle
    a: vec4f,
    
    // wave1_phase, wave2_phase, wave1_y_influence, wave2_y_influence
    b: vec4f,
    
    // pattern_mix, type mix, threshold, unused
    c: vec4f,

    // curve_freq_x, curve_freq_y, wave_distort, smoothing
    d: vec4f,
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
    let w = params.resolution.x;
    let h = params.resolution.y;
    let aspect = w / h;
    
    var p = position;
    p.x *= aspect;

    let freq_scale = 10.0;

    let freq1 = 1.0 + params.a.x * freq_scale;
    let angle1 = params.a.y * TAU;
    let phase1 = params.b.x;
    let y_influence1 = params.b.z;
    
    let freq2 = 1.0 + params.a.z * freq_scale;
    let angle2 = params.a.w * TAU;
    let phase2 = params.b.y;
    let y_influence2 = params.b.w;
    
    let pattern_mix = params.c.x;
    let pattern_type_mix = params.c.y;
    let threshold = params.c.z;
    
    let curve_freq_x = params.d.x * 10.0; 
    let curve_freq_y = params.d.y * 10.0;
    let wave_distort = params.d.z;
    let smoothing = params.d.w;
    
    let rot1 = mat2x2f(
        cos(angle1), -sin(angle1),
        sin(angle1), cos(angle1)
    );
    
    let rot2 = mat2x2f(
        cos(angle2), -sin(angle2),
        sin(angle2), cos(angle2)
    );
    
    let p1 = rot1 * p;
    let p2 = rot2 * p;
    
    let curve1 = sin(p1.y * y_influence1 * curve_freq_y) * 
        cos(p1.x * y_influence1 * curve_freq_x);
    let wave1x = freq1 * (p1.x + curve1 * wave_distort) + phase1;
    
    let curve2 = sin(p2.y * y_influence2 * curve_freq_y) * 
        cos(p2.x * y_influence2 * curve_freq_x);
    let wave2x = freq2 * (p2.x + curve2 * wave_distort) + phase2;
    
    let base1 = fract(wave1x);
    let harmonic1 = fract(2.0 * wave1x + curve1 * wave_distort);
    let harmonic1b = fract(3.0 * wave1x + curve1 * wave_distort * 1.5);
    
    let base2 = fract(wave2x);
    let harmonic2 = fract(2.0 * wave2x + curve2 * wave_distort);
    let harmonic2b = fract(3.0 * wave2x + curve2 * wave_distort * 1.5);
    
    let wave1 = mix(
        base1,
        mix(harmonic1, harmonic1b, pattern_type_mix),
        pattern_type_mix
    );
    
    let wave2 = mix(
        base2,
        mix(harmonic2, harmonic2b, pattern_type_mix),
        pattern_type_mix
    );
    
    let half_smooth = smoothing * 0.5;
    let square1 = smoothstep(0.5 - half_smooth, 0.5 + half_smooth, wave1);
    let square2 = smoothstep(0.5 - half_smooth, 0.5 + half_smooth, wave2);
    
    let mult_pattern = square1 * square2;
    let add_pattern = 
        smoothstep(1.0 - smoothing, 1.0 + smoothing, square1 + square2);
    let pattern = mix(mult_pattern, add_pattern, pattern_mix);
    
    let value = 
        smoothstep(threshold - half_smooth, threshold + half_smooth, pattern);
    
    return vec4f(value);
}