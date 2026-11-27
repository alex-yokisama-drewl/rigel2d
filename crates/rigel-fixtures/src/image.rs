//! Texture payloads.
//!
//! `docs/format.md`: one archive entry per distinct texture, PNG, **cropped to
//! its own content**. So what is encoded here is the crop rectangle's pixels,
//! never the authored image — the authored dimensions and the crop live in the
//! texture record in `puppet.json`.
//!
//! The patterns are deterministic functions of width, height and a seed, so a
//! rebuild reproduces the same bytes. Nothing here reads a file or a clock.

/// RGBA8 pixels for a `width` × `height` test pattern.
///
/// Fully opaque: this is the *cropped* content, and the padding a texture
/// record declares is outside it by construction.
fn pattern(width: u32, height: u32, seed: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let checker = ((x / 4 + y / 4) % 2) as u8;
            pixels.push((x * 7 + seed * 31) as u8);
            pixels.push((y * 11 + seed * 17) as u8);
            pixels.push(checker * 255);
            pixels.push(255);
        }
    }
    pixels
}

/// A PNG payload for a texture, cropped to its content.
pub fn payload(width: u32, height: u32, seed: u32) -> Vec<u8> {
    let pixels = pattern(width, height, seed);
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("PNG header");
        writer.write_image_data(&pixels).expect("PNG image data");
        writer.finish().expect("PNG finish");
    }
    out
}

/// Bytes that are present where a payload belongs and do not decode as an
/// image. The PNG signature is intact and the rest is not, so a loader that
/// sniffs the entry still has to fail on the contents — which is what
/// `rigged:texture-payload-corrupt` is for.
pub fn corrupt_payload() -> Vec<u8> {
    let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    bytes.extend_from_slice(b"this is not a PNG beyond the signature");
    bytes
}
