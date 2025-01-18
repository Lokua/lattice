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
    a: vec4f,
    b: vec4f,
    c: vec4f,
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
    let p = correct_aspect(position);

    let reduce_mix = params.b.x;
    let map_mix = params.b.y;

    // let reduced = mix(polar_reduce(p), wave_reduce(p), reduce_mix);
    // let mapped = mix(polar_map(reduced), wave_map(reduced), map_mix);

    // let reduced = mix(distort_reduce(p), wave_reduce(p), reduce_mix);
    // let mapped = mix(distort_map(reduced), wave_map(reduced), map_mix);

    let reduced = mix(fractal_reduce(p), wave_reduce(p), reduce_mix);
    // let mapped = mix(fractal_map(reduced), wave_map(reduced), map_mix);
    let mapped = mix(
        mix_min(fractal_map(reduced), wave_map(reduced)), 
        mix_max(fractal_map(reduced), wave_map(reduced)), 
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
    let a1 = params.c.x * 100.0;
    let a2 = params.c.y * 100.0;
    let a3 = params.c.z * 100.0;
    let a4 = params.c.w * 100.0;
    let d = length(p);
    let wave1 = sin(d * a2 - a1 * PI);
    let wave2 = sin(p.x * a3);
    let wave3 = sin(p.y * a4);
    return wave1 + wave2 + wave3;
}
fn wave_map(wave: f32) -> vec4f {
    let color = vec3(0.5 + (cos(wave) * 0.5));
    return vec4f(color, 1.0);
}

fn polar_reduce(p: vec2f) -> f32 {
    let freq = params.a.x * 100.0;
    let phase = params.a.y * TAU;
    let angle = atan2(p.y, p.x) + phase;
    let dist = length(p);
    let warped = sin(dist * freq + angle * 5.0);
    return warped;
}
fn polar_map(warped: f32) -> vec4f {
    return vec4f(vec3(0.5 + 0.5 * warped), 1.0);
}

fn distort_reduce(pos: vec2f) -> f32 {
    let freq = params.a.x * 100.0;
    let phase = params.a.y * TAU;
    var p = vec2f(pos);
    p *= sin(p * freq + phase);
    return length(p);
}
fn distort_map(d: f32) -> vec4f {
    let radius = params.a.z;
    return vec4f(vec3(smoothstep(0.0, radius, d)), 1.0);
}

fn fractal_reduce(pos: vec2f) -> f32 {
    let iteration_count = i32(floor(params.a.x * 10.0));
    let scale = params.a.y;
    let color_scale = params.a.z;
    var p = pos * scale;
    var color = 0.0;
    for (var i = 0; i < iteration_count; i++) {
        p = abs(p) * 2.0 - 1.0;
        let len = max(length(p), 0.001);
        color += color_scale / len;
    }
    return color / f32(iteration_count);
}
fn fractal_map(color_value: f32) -> vec4f {
    return vec4f(vec3f(color_value), 1.0);
}

fn modulo(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}
