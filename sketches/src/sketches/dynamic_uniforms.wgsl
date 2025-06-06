const PI: f32 = 3.14159265359;
const TAU: f32 = 6.283185307179586;

const ORIGIN_CENTER: f32 = 0.0;
const ORIGIN_TOP_RIGHT: f32 = 1.0;
const ORIGIN_BOTTOM_RIGHT: f32 = 2.0;
const ORIGIN_BOTTOM_LEFT: f32 = 3.0;
const ORIGIN_TOP_LEFT: f32 = 4.0;
const ORIGIN_ANIMATED: f32 = 5.0;
const ORIGIN_ANIMATED_Y: f32 = 6.0;

struct VertexInput {
    @location(0) position: vec2f,
};

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) pos: vec2f,
};

struct Params {
    a: vec4f,
    b: vec4f,
    c: vec4f,
    d: vec4f,
    e: vec4f,
    f: vec4f,
    g: vec4f,
    h: vec4f,
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
    let wave_phase = params.b.x;
    let wave_dist = params.b.y;
    let wave_x_freq = params.b.z;
    var wave_y_freq = params.b.w;
    let color_amt = params.c.x;
    let color_freq = params.c.y;
    let color_phase = params.c.z;
    let r_amp = params.c.w;
    let g_amp = params.d.x;
    let b_amp = params.d.y;
    let color_shift = params.d.z;
    let color_invert = params.d.w;
    let wave_phase_animation = params.e.x;
    let link_axes = params.e.y;
    let origin = params.e.z;
    let origin_offset = params.e.w;
    let grain_size = params.f.x;
    let angle_mult = params.f.y;
    let distance_mix = params.f.z;
    let color_steps = params.f.w;
    let origin_anim_x = params.g.x;
    let origin_anim_y = params.g.y;
    let p2_x = params.g.z;
    let p2_y = params.g.w;
    let film_grain_amt = params.h.x;
    let bg_alpha = params.h.y;

    if link_axes == 1.0 {
        wave_y_freq = wave_x_freq;
    }

    let p = correct_aspect(position * 0.5);

    let offs = origin_offset;
    var center: vec2f;

    if origin == ORIGIN_CENTER {
        center = vec2f(0.0);
    } else if origin == ORIGIN_TOP_RIGHT {
        center = vec2f(1.0 - offs);
    } else if origin == ORIGIN_BOTTOM_RIGHT {
        center = vec2f(1.0 - offs, -1.0 + offs);
    } else if origin == ORIGIN_BOTTOM_LEFT {
        center = vec2f(-1.0 + offs);
    } else if origin == ORIGIN_TOP_LEFT {
        center = vec2f(-1.0 + offs, 1.0 - offs);
    } else if origin == ORIGIN_ANIMATED {
        center = vec2f(origin_anim_x * offs, origin_anim_y * offs);
    } else if origin == ORIGIN_ANIMATED_Y {
        center = vec2f(0.0, origin_anim_y * offs);
    }

    let cp = p - center;
    let cp_len = length(cp);
    var d = abs(
        concentric_waves(
            cp.x, 
            cp.y, 
            sin(cp.x * p2_x) * cp_len, 
            tan(cp.y * p2_y) * cp_len, 
            grain_size, 
            angle_mult
        )
    );
    d = mix(cp_len, d, distance_mix);

    var phase: f32;
    if wave_phase_animation == 1.0 {
        phase = wave_phase;
    } else {
        phase = 0.0;
    }

    let wave1 = 
        cos(d * wave_dist - phase) + 0.5 * 
        cos((d * wave_dist - phase) * 3.0);

    let wave2 = 
        cos(p.x * wave_x_freq) + 0.5 * 
        cos((p.x * wave_x_freq) * 2.0);

    let wave3 = sin(p.y * wave_y_freq);
    
    let waves = (wave1 + wave2 + wave3);

    var c_val = ease(0.5 + (cos(waves) * 0.5));
    let color_wave = waves * color_freq + color_phase;
    
    let r = c_val + r_amp * sin(color_wave) * color_amt;
    let g = c_val + g_amp * sin(color_wave + color_shift) * color_amt;
    let b = c_val + b_amp * sin(color_wave + color_shift * 2.0) * color_amt;

    var color = vec3f(r, g, b);
    if color_invert == 1.0 {
        color = 1.0 - color;
    }

    color = vec3f(
        floor(color.r * color_steps) / color_steps, 
        floor(color.g * color_steps) / color_steps, 
        floor(color.b * color_steps) / color_steps
    );

    color = film_grain(color, p, film_grain_amt);

    return vec4f(color, bg_alpha);
}

fn correct_aspect(position: vec2f) -> vec2f {
    let w = params.a.x;
    let h = params.a.y;
    let aspect = w / h;
    var p = position;
    p.x *= aspect;
    return p;
}

fn ease(t: f32) -> f32 {
    return t * t * t;
}

fn custom_distance(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    grain_size: f32,
    angle_mult: f32,
) -> f32 {
    let distance = sqrt(pow(x2 - x1, 2.0) + pow(y2 - y1, 2.0));
    let angle = atan2(y2 - y1, x2 - x1);
    return sin(distance / grain_size) * distance * 0.5 + 
        sin(angle * 10.0) * angle_mult;
}

fn concentric_waves(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    frequency: f32,
    decay: f32,
) -> f32 {
    let distance = sqrt(pow(x2 - x1, 2.0) + pow(y2 - y1, 2.0));
    return abs(sin(distance * frequency)) * exp(-distance * decay);
}

fn hsl_to_rgb(hsl: vec3f) -> vec3f {
    let h = hsl.x;
    let s = hsl.y;
    let l = hsl.z;
    
    let c = (1.0 - abs(2.0 * l - 1.0)) * s;
    let x = c * (1.0 - abs(fract(h * 6.0) - 3.0 - 1.0));
    let m = l - c / 2.0;
    
    var rgb: vec3f;
    if (h < 1.0/6.0) {
        rgb = vec3f(c, x, 0.0);
    } else if (h < 2.0 / 6.0) {
        rgb = vec3f(x, c, 0.0);
    } else if (h < 3.0 / 6.0) {
        rgb = vec3f(0.0, c, x);
    } else if (h < 4.0 / 6.0) {
        rgb = vec3f(0.0, x, c);
    } else if (h < 5.0 / 6.0) {
        rgb = vec3f(x, 0.0, c);
    } else {
        rgb = vec3f(c, 0.0, x);
    }
    
    return rgb + m;
}

fn random_v2(p: vec2f) -> f32 {
    return fract(sin(dot(p, vec2f(12.9898, 78.233))) * 43758.5453);
}

fn film_grain(color: vec3f, p: vec2f, intensity: f32) -> vec3f {
    let random = random_v2(p);
    return clamp(color + (random - 0.5) * intensity, vec3f(0.0), vec3f(1.0));
}

