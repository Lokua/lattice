// Just copy these as needed into other sketches

// --- UTILS

fn random_v2(p: vec2f) -> f32 {
    return fract(sin(dot(p, vec2f(12.9898, 78.233))) * 43758.5453);
}

// wgsl % operator is a remainder operator, not modulo
fn modulo(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}
fn mod_v2(x: vec2f, y: vec2f) -> vec2f {
    return x - y * floor(x / y);
}
fn mod_v3(x: vec3f, y: vec3f) -> vec3f {
    return x - y * floor(x / y);
}
fn mod_v4(x: vec4f, y: vec4f) -> vec4f {
    return x - y * floor(x / y);
}

fn powf(x: f32, y: f32) -> f32 {
    return sign(x) * exp(log(abs(x)) * y);
}


fn mix_additive(c1: vec4f, c2: vec4f) -> vec4f {
    return clamp(c1 + c2, vec4f(0.0), vec4f(1.0));
}
fn mix_subtractive(c1: vec4f, c2: vec4f) -> vec4f {
    return clamp(c1 * c2, vec4f(0.0), vec4f(1.0));
}
fn mix_multiply(c1: vec4f, c2: vec4f) -> vec4f {
    return c1 * c2;
}
fn mix_screen(c1: vec4f, c2: vec4f) -> vec4f {
    return 1.0 - (1.0 - c1) * (1.0 - c2);
}
fn mix_overlay(c1: vec4f, c2: vec4f) -> vec4f {
    return vec4f(
        select(
            2.0 * c1.rgb * c2.rgb, 
            1.0 - 2.0 * (1.0 - c1.rgb) * (1.0 - c2.rgb), 
            c1.rgb <= vec3(0.5)
        ),
        c1.a
    );
}
fn mix_max(c1: vec4f, c2: vec4f) -> vec4f {
    return max(c1, c2);
}
fn mix_min(c1: vec4f, c2: vec4f) -> vec4f {
    return min(c1, c2);
}
// This can't be right
fn mix_hue_shift(c1: vec4f, c2: vec4f, t: f32) -> vec4f {
    let h1 = atan2(c1.g - c1.b, c1.r - c1.g);
    let h2 = atan2(c2.g - c2.b, c2.r - c2.g);
    let new_hue = mix(h1, h2, t);

    let len1 = length(vec3(c1.r, c1.g, c1.b));
    return vec4f(
        vec3(len1 * cos(new_hue), len1 * sin(new_hue), c1.b),
        c1.a
    );
}
fn mix_average(c1: vec4f, c2: vec4f) -> vec4f {
    return (c1 + c2) / 2.0;
}
fn mix_dodge(c1: vec4f, c2: vec4f) -> vec4f {
    return clamp(c1 / (1.0 - c2), vec4f(0.0), vec4f(1.0));
}
fn mix_burn(c1: vec4f, c2: vec4f) -> vec4f {
    return 1.0 - clamp((1.0 - c1) / c2, vec4f(0.0), vec4f(1.0));
}
fn mix_alpha(c1: vec4f, c2: vec4f, t: f32) -> vec4f {
    let blended_color = mix(c1.rgb, c2.rgb, t);
    let blended_alpha = mix(c1.a, c2.a, t);
    return vec4f(blended_color, blended_alpha);
}

// --- POST PROCESSING

fn film_grain(color: vec3f, p: vec2f, intensity: f32) -> vec3f {
    let random = random2(p);
    return clamp(color + (random - 0.5) * intensity, vec3f(0.0), vec3f(1.0));
}

fn glitch_blocks(
    color: vec3f, 
    p: vec2f, 
    block_size: f32, 
    intensity: f32
) -> vec3f {
    let block = floor(p * block_size);
    let noise = fract(sin(dot(block, vec2f(12.9898, 78.233))) * 43758.5453);
    return mix(color, vec3f(1.0) - color, step(1.0 - intensity, noise));
}
