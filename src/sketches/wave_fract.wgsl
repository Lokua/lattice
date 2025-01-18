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
    // w, h, ..unused
    resolution: vec4f,

    // wave_phase, wave_radial_freq, wave_horiz_freq, wave_vert_freq
    a: vec4f,

    // fract_count, fract_scale, fract_color_scale, wave_power
    b: vec4f,

    // reduce_mix, map_mix, wave_bands, wave_threshold
    c: vec4f,

    // fract_contrast, fract_steps, ..unused
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
    let reduce_mix = params.c.x;
    let map_mix = params.c.y;

    let p = correct_aspect(position);

    let reduced = mix(wave_reduce(p), fractal_reduce(p), reduce_mix);

    let mapped = mix(
        mix_min(wave_map(reduced), fractal_map(reduced)), 
        mix_max(wave_map(reduced), fractal_map(reduced)), 
        map_mix
    );

    return mapped;
}

fn correct_aspect(position: vec2f) -> vec2f {
    let w = params.resolution.x;
    let h = params.resolution.y;

    let aspect = w / h;
    var p = position;
    p.x *= aspect;
    
    return p;
}

fn wave_reduce(p: vec2f) -> f32 {
    let phase = params.a.x;
    let radial_freq = params.a.y * 100.0;
    let horiz_freq = params.a.z * 100.0;
    let vert_freq = params.a.w * 100.0;
    let power = params.b.w * 10.0;

    let d = length(p);
    let wave1 = sin(d * radial_freq - phase * TAU);
    let wave2 = sin(p.x * horiz_freq);
    let wave3 = sin(p.y * vert_freq);

    return powf(wave1 + wave2 + wave3, power);
}

fn wave_map(wave: f32) -> vec4f {
    let n_bands = floor(params.c.z);
    let threshold = params.c.w;

    let angle = wave * TAU;
    let band_angle = floor(angle * n_bands) / n_bands;
    let value = cos(band_angle);
    
    let thresholded = step(threshold, value);
    
    let normalized = (value + 1.0) * 0.5 * thresholded;
    let result = floor(normalized * n_bands) / (n_bands - 1.0);
    
    return vec4f(vec3f(result), 1.0);
}

fn fractal_reduce(pos: vec2f) -> f32 {
    let count = params.b.x * 10.0; 
    let scale = params.b.y;
    let color_scale = params.b.z;
    
    var p = pos * scale;
    var color = 0.0;
    let MAX_ITERATIONS = 20;
    
    for (var i = 0; i < MAX_ITERATIONS; i++) {
        let weight = 1.0 - smoothstep(count - 1.0, count, f32(i));
        if (weight <= 0.0) { break; }
        
        p = abs(p) * 2.0 - 1.0;
        let len = max(length(p), 0.001);
        color += (color_scale / len) * weight;
    }
    
    return color / count;
}

fn fractal_map(color_value: f32) -> vec4f {
    let contrast = params.d.x;
    let steps = params.d.y;
    let contrasted = pow(color_value, contrast);
    let stepped = floor(contrasted * steps) / steps;
    return vec4f(vec3f(stepped), 1.0);
}

fn mix_min(c1: vec4f, c2: vec4f) -> vec4f {
    return min(c1, c2);
}

fn mix_max(c1: vec4f, c2: vec4f) -> vec4f {
    return max(c1, c2);
}

fn powf(x: f32, y: f32) -> f32 {
    return sign(x) * exp(log(abs(x)) * y);
}

