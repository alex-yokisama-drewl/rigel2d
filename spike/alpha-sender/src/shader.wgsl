// The test frame: six vertical bands, described in REFERENCE.md.
//
// Values are written as n/255 so that each lands exactly on an 8-bit code with no rounding, and
// the target is a linear (non-sRGB) format so that what is written is what is stored. Both are
// load-bearing: see REFERENCE.md, "The trap this table is built to avoid".

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    // Fullscreen triangle. No vertex buffer, no index buffer.
    let x = f32(i32(i) / 2) * 4.0 - 1.0;
    let y = f32(i32(i) & 1) * 4.0 - 1.0;
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let band = i32(pos.x) / 128;

    // Straight (non-premultiplied) colour and alpha, per band.
    var rgb: vec3<f32>;
    var a: f32;

    switch band {
        case 0: { rgb = vec3<f32>(1.0, 1.0, 1.0); a = 255.0 / 255.0; } // P0 opaque white
        case 1: { rgb = vec3<f32>(1.0, 1.0, 1.0); a = 128.0 / 255.0; } // P1 white, half
        case 2: { rgb = vec3<f32>(1.0, 0.0, 0.0); a =  64.0 / 255.0; } // P2 red, quarter
        case 3: { rgb = vec3<f32>(0.0, 1.0, 0.0); a = 192.0 / 255.0; } // P3 green, three-quarter
        case 4: { rgb = vec3<f32>(0.0, 0.0, 0.0); a =   0.0 / 255.0; } // P4 fully transparent
        default: { rgb = vec3<f32>(0.0, 0.0, 0.0); a = 255.0 / 255.0; } // P5 opaque black
    }

    // PREMULTIPLY is specialised at pipeline creation by string substitution on this file, so
    // that both variants are built from one source and cannot drift apart. The sender emits
    // premultiplied by default, because that is what the PipeWire dmabuf contract specifies and
    // what OBS expects from a texture it imports with alpha.
    //
    // Emitting the straight variant on purpose is how we tell a route that *converts* from one
    // that merely *passes through*: a pass-through route scored against REFERENCE.md will report
    // whichever the sender actually sent.
    return vec4<f32>(rgb * PREMULTIPLY_FACTOR, a);
}
