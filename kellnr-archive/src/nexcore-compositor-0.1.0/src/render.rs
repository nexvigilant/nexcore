// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Render target abstraction — software and GPU rendering backends.
//!
//! ## Primitive Grounding
//!
//! - μ Mapping: Surface framebuffers → display output
//! - σ Sequence: Render pipeline stages (clear → decorate → composite → present)
//! - ∂ Boundary: Clipping regions for surface rendering
//! - Σ Sum: Aggregate multiple surfaces into single output

use crate::decoration::{DecorationFrame, DecorationRenderer};
use crate::surface::{Rect, Surface, Visibility};

/// A render command in the compositor pipeline.
///
/// Tier: T2-C (μ + σ — mapped pipeline operation)
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// Clear the display to a solid color.
    Clear([u8; 4]),

    /// Blit a surface's framebuffer to the display.
    BlitSurface {
        /// Surface bounds on screen.
        bounds: Rect,
        /// RGBA framebuffer data.
        framebuffer: Vec<u8>,
    },

    /// Blit a decoration frame to the display.
    BlitDecoration {
        /// Decoration frame data.
        frame: DecorationFrame,
    },

    /// Present the composited frame to the display.
    Present,
}

/// Render pipeline — generates an ordered list of render commands.
///
/// Tier: T3 (μ + σ + ∂ + Σ — full render pipeline)
///
/// Separates the "what to render" from "how to render" so the
/// same pipeline can drive both software blitting and GPU rendering.
pub struct RenderPipeline {
    /// Decoration renderer for desktop mode.
    decorations: DecorationRenderer,
    /// Whether decorations are enabled (desktop mode only).
    decorations_enabled: bool,
}

impl RenderPipeline {
    /// Create a new render pipeline.
    pub fn new(decorations_enabled: bool) -> Self {
        Self {
            decorations: DecorationRenderer::new(),
            decorations_enabled,
        }
    }

    /// Get the decoration renderer.
    pub fn decoration_renderer(&self) -> &DecorationRenderer {
        &self.decorations
    }

    /// Whether decorations are enabled.
    pub const fn decorations_enabled(&self) -> bool {
        self.decorations_enabled
    }

    /// Generate render commands for a frame.
    ///
    /// Takes surfaces in z-order (back to front) and produces
    /// an ordered list of render commands.
    pub fn generate_commands(
        &self,
        surfaces: &[&Surface],
        background: [u8; 4],
    ) -> Vec<RenderCommand> {
        let mut commands = Vec::with_capacity(surfaces.len() * 2 + 2);

        // 1. Clear
        commands.push(RenderCommand::Clear(background));

        // 2. Render surfaces back-to-front
        for surface in surfaces {
            if surface.visibility != Visibility::Visible {
                continue;
            }

            // Decorations first (behind surface content)
            if self.decorations_enabled {
                let frame =
                    self.decorations
                        .render(&surface.bounds, &surface.title, surface.focused);
                commands.push(RenderCommand::BlitDecoration { frame });
            }

            // Surface content
            if let Some(ref fb) = surface.framebuffer {
                commands.push(RenderCommand::BlitSurface {
                    bounds: surface.bounds,
                    framebuffer: fb.clone(),
                });
            }
        }

        // 3. Present
        commands.push(RenderCommand::Present);

        commands
    }

    /// Execute render commands against a software framebuffer.
    ///
    /// This is the CPU-based rendering path used for testing
    /// and when no GPU is available.
    pub fn execute_software(
        &self,
        commands: &[RenderCommand],
        framebuffer: &mut [u8],
        resolution: (u32, u32),
    ) {
        for cmd in commands {
            match cmd {
                RenderCommand::Clear(color) => {
                    for pixel in framebuffer.chunks_exact_mut(4) {
                        pixel[0] = color[0];
                        pixel[1] = color[1];
                        pixel[2] = color[2];
                        pixel[3] = color[3];
                    }
                }
                RenderCommand::BlitSurface {
                    bounds,
                    framebuffer: src,
                } => {
                    blit_rgba(framebuffer, resolution, src, bounds);
                }
                RenderCommand::BlitDecoration { frame } => {
                    blit_rgba(
                        framebuffer,
                        resolution,
                        &frame.framebuffer,
                        &frame.outer_bounds,
                    );
                }
                RenderCommand::Present => {
                    // Software path: framebuffer is already the output
                }
            }
        }
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Blit RGBA source data into a destination framebuffer with alpha compositing.
///
/// Handles clipping at display boundaries and per-pixel alpha blending.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::cast_lossless
)]
fn blit_rgba(dst: &mut [u8], resolution: (u32, u32), src: &[u8], bounds: &Rect) {
    let src_stride = bounds.width as usize * 4;
    let (display_w, display_h) = resolution;

    for row in 0..bounds.height {
        let dst_y = bounds.y + row as i32;
        if dst_y < 0 || dst_y >= display_h as i32 {
            continue;
        }

        let src_offset = row as usize * src_stride;
        if src_offset + src_stride > src.len() {
            continue;
        }

        for col in 0..bounds.width {
            let dst_x = bounds.x + col as i32;
            if dst_x < 0 || dst_x >= display_w as i32 {
                continue;
            }

            let src_px = src_offset + col as usize * 4;
            let dst_idx = (dst_y as usize * display_w as usize + dst_x as usize) * 4;

            if src_px + 3 < src.len() && dst_idx + 3 < dst.len() {
                let alpha = u16::from(src[src_px + 3]);
                if alpha == 255 {
                    dst[dst_idx] = src[src_px];
                    dst[dst_idx + 1] = src[src_px + 1];
                    dst[dst_idx + 2] = src[src_px + 2];
                    dst[dst_idx + 3] = 255;
                } else if alpha > 0 {
                    let inv_alpha = 255 - alpha;
                    for c in 0..3 {
                        let s = u16::from(src[src_px + c]);
                        let d = u16::from(dst[dst_idx + c]);
                        dst[dst_idx + c] = ((s * alpha + d * inv_alpha) / 255) as u8;
                    }
                    dst[dst_idx + 3] = 255;
                }
            }
        }
    }
}

/// Frame statistics from the render pipeline.
///
/// Tier: T2-P (N Quantity — render metrics)
#[derive(Debug, Clone, Copy, Default)]
pub struct FrameStats {
    /// Number of render commands in this frame.
    pub command_count: usize,
    /// Number of surfaces rendered.
    pub surface_count: usize,
    /// Number of decoration frames rendered.
    pub decoration_count: usize,
    /// Total pixels blitted (approximate).
    pub pixels_blitted: u64,
}

impl FrameStats {
    /// Compute stats from a command list.
    pub fn from_commands(commands: &[RenderCommand]) -> Self {
        let mut stats = Self {
            command_count: commands.len(),
            ..Default::default()
        };

        for cmd in commands {
            match cmd {
                RenderCommand::BlitSurface { bounds, .. } => {
                    stats.surface_count += 1;
                    stats.pixels_blitted += bounds.area();
                }
                RenderCommand::BlitDecoration { frame } => {
                    stats.decoration_count += 1;
                    stats.pixels_blitted += frame.outer_bounds.area();
                }
                _ => {}
            }
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::SurfaceId;

    fn make_surface(id: u32, x: i32, y: i32, w: u32, h: u32, color: [u8; 4]) -> Surface {
        let mut s = Surface::new(
            SurfaceId::new(id),
            format!("app-{id}"),
            Rect::new(x, y, w, h),
        );
        s.allocate_framebuffer();
        s.fill(color[0], color[1], color[2], color[3]);
        s
    }

    #[test]
    fn pipeline_generates_commands() {
        let pipeline = RenderPipeline::new(false);
        let s1 = make_surface(1, 0, 0, 10, 10, [255, 0, 0, 255]);
        let surfaces: Vec<&Surface> = vec![&s1];

        let commands = pipeline.generate_commands(&surfaces, [0, 0, 0, 255]);

        // Should be: Clear, BlitSurface, Present
        assert_eq!(commands.len(), 3);
        assert!(matches!(commands[0], RenderCommand::Clear(_)));
        assert!(matches!(commands[1], RenderCommand::BlitSurface { .. }));
        assert!(matches!(commands[2], RenderCommand::Present));
    }

    #[test]
    fn pipeline_with_decorations() {
        let pipeline = RenderPipeline::new(true);
        let mut s1 = make_surface(1, 10, 40, 100, 80, [0, 0, 255, 255]);
        s1.title = "Test".into();
        s1.focused = true;
        let surfaces: Vec<&Surface> = vec![&s1];

        let commands = pipeline.generate_commands(&surfaces, [0, 0, 0, 255]);

        // Should be: Clear, BlitDecoration, BlitSurface, Present
        assert_eq!(commands.len(), 4);
        assert!(matches!(commands[0], RenderCommand::Clear(_)));
        assert!(matches!(commands[1], RenderCommand::BlitDecoration { .. }));
        assert!(matches!(commands[2], RenderCommand::BlitSurface { .. }));
        assert!(matches!(commands[3], RenderCommand::Present));
    }

    #[test]
    fn pipeline_skips_hidden_surfaces() {
        let pipeline = RenderPipeline::new(false);
        let mut s1 = make_surface(1, 0, 0, 10, 10, [255, 0, 0, 255]);
        s1.visibility = Visibility::Hidden;
        let surfaces: Vec<&Surface> = vec![&s1];

        let commands = pipeline.generate_commands(&surfaces, [0, 0, 0, 255]);

        // Should be: Clear, Present (no blit for hidden surface)
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn software_execution_clears() {
        let pipeline = RenderPipeline::new(false);
        let bg = [30, 30, 50, 255];
        let commands = vec![RenderCommand::Clear(bg), RenderCommand::Present];

        let mut fb = vec![0u8; 4 * 4 * 4]; // 4x4
        pipeline.execute_software(&commands, &mut fb, (4, 4));

        assert_eq!(fb[0], 30);
        assert_eq!(fb[1], 30);
        assert_eq!(fb[2], 50);
        assert_eq!(fb[3], 255);
    }

    #[test]
    fn software_execution_blits_surface() {
        let pipeline = RenderPipeline::new(false);
        let s = make_surface(1, 0, 0, 4, 4, [255, 0, 0, 255]);

        let surfaces: Vec<&Surface> = vec![&s];
        let commands = pipeline.generate_commands(&surfaces, [0, 0, 0, 255]);

        let mut fb = vec![0u8; 4 * 4 * 4];
        pipeline.execute_software(&commands, &mut fb, (4, 4));

        // Should be red
        assert_eq!(fb[0], 255);
        assert_eq!(fb[1], 0);
        assert_eq!(fb[2], 0);
    }

    #[test]
    fn frame_stats_counts() {
        let pipeline = RenderPipeline::new(true);
        let mut s1 = make_surface(1, 0, 30, 100, 80, [0, 255, 0, 255]);
        s1.title = "Stats".into();
        let surfaces: Vec<&Surface> = vec![&s1];

        let commands = pipeline.generate_commands(&surfaces, [0, 0, 0, 255]);
        let stats = FrameStats::from_commands(&commands);

        assert_eq!(stats.surface_count, 1);
        assert_eq!(stats.decoration_count, 1);
        assert!(stats.pixels_blitted > 0);
    }

    #[test]
    fn blit_clips_at_edges() {
        let pipeline = RenderPipeline::new(false);
        // Surface extends beyond display boundary
        let s = make_surface(1, -2, -2, 6, 6, [128, 128, 128, 255]);
        let surfaces: Vec<&Surface> = vec![&s];
        let commands = pipeline.generate_commands(&surfaces, [0, 0, 0, 255]);

        let mut fb = vec![0u8; 4 * 4 * 4]; // 4x4 display
        pipeline.execute_software(&commands, &mut fb, (4, 4));

        // Pixel (0,0) should be gray (from the clipped surface)
        assert_eq!(fb[0], 128);
    }

    #[test]
    fn empty_surface_list() {
        let pipeline = RenderPipeline::new(false);
        let surfaces: Vec<&Surface> = vec![];
        let commands = pipeline.generate_commands(&surfaces, [10, 20, 30, 255]);

        // Just Clear + Present
        assert_eq!(commands.len(), 2);

        let stats = FrameStats::from_commands(&commands);
        assert_eq!(stats.surface_count, 0);
        assert_eq!(stats.decoration_count, 0);
    }
}
