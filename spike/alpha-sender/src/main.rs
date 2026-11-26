//! S0 alpha sender — the instrument for the transparency spike.
//!
//! Renders the test frame described in `REFERENCE.md` and reports the probe values, so that each
//! candidate route into OBS is scored against numbers fixed before any route was tried.
//!
//! Throwaway scaffolding. See `docs/wip/s0-transparency-spike.md`.

mod window;

pub const WIDTH: u32 = 768;
pub const HEIGHT: u32 = 256;
const BAND: u32 = 128;

/// Probes, as `REFERENCE.md` fixes them. `x = 64 + 128 i`, `y = 128`.
const PROBES: [(&str, u32, u32); 6] = [
    ("P0", 64, 128),
    ("P1", 192, 128),
    ("P2", 320, 128),
    ("P3", 448, 128),
    ("P4", 576, 128),
    ("P5", 704, 128),
];

/// Expected RGBA at each probe, from `REFERENCE.md`. `None` means the table states the value is
/// undefined for that interpretation.
const EXPECT_PREMULTIPLIED: [Option<[u8; 4]>; 6] = [
    Some([255, 255, 255, 255]),
    Some([128, 128, 128, 128]),
    Some([64, 0, 0, 64]),
    Some([0, 192, 0, 192]),
    Some([0, 0, 0, 0]),
    Some([0, 0, 0, 255]),
];

const EXPECT_STRAIGHT: [Option<[u8; 4]>; 6] = [
    Some([255, 255, 255, 255]),
    Some([255, 255, 255, 128]),
    Some([255, 0, 0, 64]),
    Some([0, 255, 0, 192]),
    None, // P4: RGB undefined when A is zero
    Some([0, 0, 0, 255]),
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let straight = args.iter().any(|a| a == "--straight");

    let out = args
        .iter()
        .position(|a| a == "--dump")
        .and_then(|i| args.get(i + 1))
        .cloned();

    if args.iter().any(|a| a == "--window") {
        window::run(
            straight,
            args.iter().any(|a| a == "--srgb"),
            args.iter().any(|a| a == "--solo"),
        );
        return;
    }

    match out {
        Some(path) => pollster::block_on(dump(&path, straight)),
        None => {
            eprintln!(
                "usage: alpha-sender (--dump <path.png> | --window) [--straight]\n\
                 \n\
                 --dump      render offscreen with no window, write the PNG, print the probes\n\
                 --window    show the test frame over an opaque backdrop, for the compositor test\n\
                 --straight  emit straight alpha instead of premultiplied\n\
                 \n\
                 The windowed and OBS-facing modes arrive with their own tasks; the dump exists\n\
                 first so that a later failure is attributable to a route rather than to this."
            );
            std::process::exit(2);
        }
    }
}


/// Builds the test-frame pipeline. Shared by the offscreen dump and the windowed path so the two
/// cannot drift: a difference between them would be indistinguishable from a compositor fault.
pub fn build_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    straight: bool,
) -> wgpu::RenderPipeline {
    let source = include_str!("shader.wgsl")
        .replace("PREMULTIPLY_FACTOR", if straight { "1.0" } else { "a" });

    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("test frame"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("test frame"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            // No blending: the frame is written, not composited. Blending here would mix the
            // bands with the clear colour and every probe would be wrong in a plausible way.
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    })
}

async fn dump(path: &str, straight: bool) {
    let instance = wgpu::Instance::default();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .expect("no GPU adapter — the spike needs a real one, not a software fallback");

    let info = adapter.get_info();
    eprintln!(
        "adapter: {} ({:?}, {:?})",
        info.name, info.device_type, info.backend
    );

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("alpha-sender"),
            ..Default::default()
        })
        .await
        .expect("could not open the device");

    // Linear, never sRGB. REFERENCE.md explains why every number in the table depends on it.
    let format = wgpu::TextureFormat::Bgra8Unorm;

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test frame"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let pipeline = build_pipeline(&device, format, straight);

    // Row stride is already 256-aligned at this width; REFERENCE.md explains the choice.
    let bytes_per_row = WIDTH * 4;
    assert_eq!(bytes_per_row % 256, 0, "readback stride must need no padding");

    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: (bytes_per_row * HEIGHT) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("test frame"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.draw(0..3, 0..1);
    }

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(HEIGHT),
            },
        },
        wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
    );

    queue.submit([encoder.finish()]);

    let slice = readback.slice(..);
    slice.map_async(wgpu::MapMode::Read, |r| r.expect("readback map failed"));
    device
        .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
        .expect("poll failed");

    // BGRA on the wire; reorder to RGBA so the report matches REFERENCE.md.
    let bgra = slice.get_mapped_range().expect("mapped range").to_vec();
    let mut rgba = bgra.clone();
    for px in rgba.chunks_exact_mut(4) {
        px.swap(0, 2);
    }

    write_png(path, &rgba);
    report(&rgba, straight, path);
}

fn write_png(path: &str, rgba: &[u8]) {
    let file = std::fs::File::create(path).expect("could not create the output file");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), WIDTH, HEIGHT);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .expect("png header")
        .write_image_data(rgba)
        .expect("png data");
}

fn report(rgba: &[u8], straight: bool, path: &str) {
    let expected = if straight {
        &EXPECT_STRAIGHT
    } else {
        &EXPECT_PREMULTIPLIED
    };

    println!(
        "\nwrote {path} — {} alpha, {}x{}",
        if straight { "straight" } else { "premultiplied" },
        WIDTH,
        HEIGHT
    );
    println!("probe   x,y        got              want             verdict");

    let mut all_ok = true;
    let mut rgb_exceeds_alpha = false;

    for (i, (name, x, y)) in PROBES.iter().enumerate() {
        let o = ((y * WIDTH + x) * 4) as usize;
        let got = [rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3]];

        if got[0].max(got[1]).max(got[2]) > got[3] {
            rgb_exceeds_alpha = true;
        }

        let (want, verdict) = match expected[i] {
            Some(w) if w == got => (format!("{:?}", w), "ok"),
            Some(w) => {
                all_ok = false;
                (format!("{:?}", w), "MISMATCH")
            }
            None => ("undefined".to_string(), "n/a"),
        };

        println!(
            "{name}      {x},{y}   {:?}   {:<16} {verdict}",
            got, want
        );
    }

    // REFERENCE.md: RGB above A cannot occur in valid premultiplied data, whatever a route says.
    println!(
        "\nRGB > A anywhere: {} — {}",
        rgb_exceeds_alpha,
        if rgb_exceeds_alpha {
            "data is straight, whatever it claims"
        } else {
            "consistent with premultiplied"
        }
    );

    if all_ok {
        println!("ALL PROBES MATCH the table for the requested interpretation.");
    } else {
        println!("PROBES DISAGREE with REFERENCE.md. Fix this before wiring any route.");
        std::process::exit(1);
    }

    // Bands exist so a human can also look; BAND is referenced so the constant stays honest.
    debug_assert_eq!(WIDTH / BAND, PROBES.len() as u32);
}
