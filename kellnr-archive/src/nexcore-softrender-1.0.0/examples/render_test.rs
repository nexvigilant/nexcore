//! Render test: produces a PPM image demonstrating the software rasterizer
//!
//! Usage: cargo run -p nexcore-softrender --example render_test > output.ppm

use nexcore_softrender::geometry::shapes;
use nexcore_softrender::geometry::{Mesh, Triangle, Vertex};
use nexcore_softrender::math::transform::{compose_2d, rotate_2d, scale_2d, translate_2d};
use nexcore_softrender::math::{Color, Vec2};
use nexcore_softrender::pipeline::{Framebuffer, rasterize_mesh, rasterize_mesh_transformed};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

fn main() {
    let mut fb = Framebuffer::new(WIDTH, HEIGHT);

    // Background: NexVigilant Navy
    fb.clear(Color::NAVY);

    // === Section 1: Basic shapes (top-left) ===

    // Colored rectangles
    let r1 = shapes::rect(20.0, 20.0, 120.0, 80.0, Color::ACCENT_CYAN);
    rasterize_mesh(&mut fb, &r1);

    let r2 = shapes::rect(160.0, 20.0, 120.0, 80.0, Color::ACCENT_GREEN);
    rasterize_mesh(&mut fb, &r2);

    let r3 = shapes::rect(300.0, 20.0, 120.0, 80.0, Color::ACCENT_GOLD);
    rasterize_mesh(&mut fb, &r3);

    let r4 = shapes::rect(440.0, 20.0, 120.0, 80.0, Color::ACCENT_RED);
    rasterize_mesh(&mut fb, &r4);

    // === Section 2: Gradient rectangle (top-right) ===
    let grad = shapes::rect_gradient(
        600.0,
        20.0,
        180.0,
        80.0,
        Color::ACCENT_CYAN,
        Color::ACCENT_GREEN,
        Color::ACCENT_GOLD,
        Color::ACCENT_RED,
    );
    rasterize_mesh(&mut fb, &grad);

    // === Section 3: Circles (middle row) ===
    let c1 = shapes::circle(80.0, 180.0, 50.0, 64, Color::ACCENT_CYAN);
    rasterize_mesh(&mut fb, &c1);

    let c2 = shapes::circle(220.0, 180.0, 50.0, 6, Color::ACCENT_GREEN); // hexagon
    rasterize_mesh(&mut fb, &c2);

    let c3 = shapes::circle(360.0, 180.0, 50.0, 3, Color::ACCENT_GOLD); // triangle
    rasterize_mesh(&mut fb, &c3);

    let c4 = shapes::circle(500.0, 180.0, 50.0, 32, Color::ACCENT_RED);
    rasterize_mesh(&mut fb, &c4);

    // === Section 4: Lines (middle) ===
    for i in 0..8 {
        let angle = (i as f64) * core::f64::consts::PI / 8.0;
        let cx = 680.0;
        let cy = 180.0;
        let r = 50.0;
        let x1 = cx + r * angle.cos();
        let y1 = cy + r * angle.sin();
        let color = Color::rgba(i as f64 / 8.0, 1.0 - (i as f64 / 8.0), 0.5, 1.0);
        let l = shapes::line(cx, cy, x1, y1, 2.0, color);
        rasterize_mesh(&mut fb, &l);
    }

    // === Section 5: Rounded rectangles (lower-middle) ===
    let rr1 = shapes::rounded_rect(20.0, 260.0, 160.0, 80.0, 15.0, Color::ACCENT_CYAN, 8);
    rasterize_mesh(&mut fb, &rr1);

    let rr2 = shapes::rounded_rect(200.0, 260.0, 160.0, 80.0, 30.0, Color::ACCENT_GREEN, 8);
    rasterize_mesh(&mut fb, &rr2);

    let rr3 = shapes::rounded_rect(380.0, 260.0, 160.0, 80.0, 40.0, Color::ACCENT_GOLD, 16);
    rasterize_mesh(&mut fb, &rr3);

    // === Section 6: Barycentric color interpolation (RGB triangle) ===
    let tri_mesh = Mesh {
        triangles: vec![Triangle::new(
            Vertex::colored(620.0, 260.0, Color::RED),
            Vertex::colored(780.0, 260.0, Color::GREEN),
            Vertex::colored(700.0, 340.0, Color::BLUE),
        )],
    };
    rasterize_mesh(&mut fb, &tri_mesh);

    // === Section 7: Transformed shapes (bottom) ===

    // A small square, rotated 45° and translated
    let sq = shapes::rect(0.0, 0.0, 60.0, 60.0, Color::WHITE);
    let t = compose_2d(
        &translate_2d(80.0, 430.0),
        &compose_2d(
            &rotate_2d(core::f64::consts::FRAC_PI_4),
            &translate_2d(-30.0, -30.0), // center the square on origin before rotation
        ),
    );
    rasterize_mesh_transformed(&mut fb, &sq, &t);

    // Scaled circle
    let small_c = shapes::circle(0.0, 0.0, 30.0, 32, Color::ACCENT_CYAN);
    let t2 = compose_2d(
        &translate_2d(220.0, 430.0),
        &scale_2d(2.0, 1.0), // ellipse via non-uniform scale
    );
    rasterize_mesh_transformed(&mut fb, &small_c, &t2);

    // Chain of rotated rectangles (fan)
    for i in 0..12 {
        let angle = (i as f64) * core::f64::consts::PI / 6.0;
        let bar = shapes::rect(
            0.0,
            -3.0,
            40.0,
            6.0,
            Color::rgba(0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin(), 0.7, 0.8),
        );
        let t_fan = compose_2d(&translate_2d(420.0, 460.0), &rotate_2d(angle));
        rasterize_mesh_transformed(&mut fb, &bar, &t_fan);
    }

    // === Section 8: Primitive symbols (bottom-right) ===
    // Draw 16 colored squares in a 4×4 grid representing the Lex Primitiva
    let prim_colors = [
        Color::from_hex(0x00CCFF), // σ Sequence
        Color::from_hex(0x4488FF), // μ Mapping
        Color::from_hex(0x00CC66), // ς State
        Color::from_hex(0xCC66FF), // ρ Recursion
        Color::from_hex(0x666666), // ∅ Void
        Color::from_hex(0xFFCC00), // ∂ Boundary
        Color::from_hex(0x008888), // ν Frequency
        Color::from_hex(0xFFFFFF), // ∃ Existence
        Color::from_hex(0x006633), // π Persistence
        Color::from_hex(0xFF4444), // → Causality
        Color::from_hex(0xCC8800), // κ Comparison
        Color::from_hex(0x00CCFF), // N Quantity
        Color::from_hex(0x8800CC), // λ Location
        Color::from_hex(0x880000), // ∝ Irreversibility
        Color::from_hex(0x00CC66), // Σ Sum
        Color::from_hex(0xFFCC00), // × Product
    ];

    for (i, color) in prim_colors.iter().enumerate() {
        let row = i / 4;
        let col = i % 4;
        let x = 580.0 + col as f64 * 55.0;
        let y = 380.0 + row as f64 * 55.0;
        let sq = shapes::rounded_rect(x, y, 45.0, 45.0, 8.0, *color, 6);
        rasterize_mesh(&mut fb, &sq);
    }

    // === Output PPM to stdout ===
    // PPM P6: binary RGB
    let header = format!("P6\n{} {}\n255\n", WIDTH, HEIGHT);
    let mut output: Vec<u8> = Vec::with_capacity(header.len() + (WIDTH * HEIGHT * 3) as usize);
    output.extend_from_slice(header.as_bytes());

    for &pixel in &fb.pixels {
        output.push(((pixel >> 16) & 0xFF) as u8); // R
        output.push(((pixel >> 8) & 0xFF) as u8); // G
        output.push((pixel & 0xFF) as u8); // B
    }

    // Write binary to stdout
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let _ = lock.write_all(&output);
}
