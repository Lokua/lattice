const PI: f32 = 3.14159265359;

struct VertexInput {
    @location(0) position: vec2f,
};

struct VertexOutput {
    @builtin(position) position: vec4f,
    // @location(0) uv: vec2f,
    @location(0) pos: vec2f,
};

struct Params {
    resolution: vec2f,
    mode: u32,
    radius: f32,
}

@group(0) @binding(0)
var<uniform> params: Params;

@vertex
fn vs_main(vert: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Convert 2D to 4D (Z = 0.0 = no depth)
    out.position = vec4f(vert.position, 0.0, 1.0);

    // Map [-1,1] to [0,1]
    // out.uv = vert.position * 0.5 + 0.5;
    out.pos = vert.position;

    return out;
}

@fragment
// Using NDC over UV because I find UV unintuitive.
// Perhaps I'll regret this?
fn fs_main(@location(0) position: vec2f) -> @location(0) vec4f {
    let aspect = params.resolution.x / params.resolution.y;
    
    // Center the coordinates (move 0.0 to center instead of corner)
    // Now ranges from [-0.5, 0.5]
    // var centered = uv - 0.5;
    
    // Correct the x coordinate for aspect ratio.
    // For example if aspect ratio is 1.78 (1600/900),
    // this will stretch things out, but also makes it so fewer 
    // points will pass our radius check, preserving the circle's symmetry
    // centered.x *= aspect;

    var p = position;
    p.x *= aspect;
    
    let d = length(p);

    if (params.mode == 0u) {
        return vec4f(smoothstep(0.0, params.radius, d));
    }
    
    return vec4f(step(d, params.radius));
}