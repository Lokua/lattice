// Just copy these as needed into other sketches

// --- UTILS

fn random2(p: vec2f) -> f32 {
    return fract(sin(dot(p, vec2f(12.9898, 78.233))) * 43758.5453);
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
