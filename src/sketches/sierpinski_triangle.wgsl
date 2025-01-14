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

    // iterations, scale, y_offset, unused
    a: vec4f,          
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
    let iterations = i32(params.a.x);
    let scale = params.a.y;
    let y_offset = params.a.z;

    let aspect = params.resolution.x / params.resolution.y;
    var p = position;
    p.x *= aspect;
    
    p = p * scale;
    p.y = p.y + y_offset; 
    
    let s = sierpinski(p, iterations);
    return vec4f(vec3f(s), 1.0);
}

fn in_triangle(p: vec2f, a: vec2f, b: vec2f, c: vec2f) -> bool {
    // Returns true if point p is inside triangle abc
    let v0 = c - a;
    let v1 = b - a;
    let v2 = p - a;
    
    let dot00 = dot(v0, v0);
    let dot01 = dot(v0, v1);
    let dot02 = dot(v0, v2);
    let dot11 = dot(v1, v1);
    let dot12 = dot(v1, v2);
    
    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
    
    return (u >= 0.0) && (v >= 0.0) && (u + v <= 1.0);
}

fn sierpinski(p: vec2f, iterations: i32) -> f32 {
    // Define the main triangle vertices
    let top = vec2f(0.0, 1.0);
    let left = vec2f(-0.866, -0.5);  // -sqrt(3)/2, -0.5
    let right = vec2f(0.866, -0.5);  // sqrt(3)/2, -0.5
    
    // Check if point is in the main triangle
    if (!in_triangle(p, top, left, right)) {
        return 0.0;
    }
    
    var current_p = p;
    for (var i = 0; i < iterations; i++) {
        // Find midpoints of the current triangle
        let mid_top = (top + right) * 0.5;
        let mid_left = (left + top) * 0.5;
        let mid_right = (right + left) * 0.5;
        
        // Check which sub-triangle the point is in
        if (in_triangle(current_p, mid_top, mid_left, mid_right)) {
            return 0.0;  // Point is in the middle (removed) triangle
        }
        
        // Scale the point relative to its sub-triangle
        if (in_triangle(current_p, top, mid_left, mid_top)) {
            current_p = (current_p - top) * 2.0 + top;
        } else if (in_triangle(current_p, mid_left, left, mid_right)) {
            current_p = (current_p - left) * 2.0 + left;
        } else {
            current_p = (current_p - right) * 2.0 + right;
        }
    }
    
    return 1.0;
}