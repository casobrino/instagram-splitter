use eframe::egui::{self, Color32, Pos2, Rect, TextureHandle, Vec2};
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

pub const OUTPUT_WIDTH: u32 = 1080;
pub const MIN_ZOOM: f32 = 0.5;
pub const MAX_ZOOM: f32 = 4.0;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum CineCanvas {
    #[default]
    Portrait45,
    Story916,
}

impl CineCanvas {
    pub fn all() -> [CineCanvas; 2] {
        [CineCanvas::Portrait45, CineCanvas::Story916]
    }

    pub fn output_size(self) -> (u32, u32) {
        match self {
            CineCanvas::Portrait45 => (OUTPUT_WIDTH, 1350),
            CineCanvas::Story916 => (OUTPUT_WIDTH, 1920),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CineCanvas::Portrait45 => "4:5",
            CineCanvas::Story916 => "9:16",
        }
    }

    pub fn size_label(self) -> &'static str {
        match self {
            CineCanvas::Portrait45 => "1080×1350",
            CineCanvas::Story916 => "1080×1920",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum CineStripCount {
    #[default]
    Three,
    Four,
}

impl CineStripCount {
    pub fn count(self) -> usize {
        match self {
            CineStripCount::Three => 3,
            CineStripCount::Four => 4,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CineStripCount::Three => "3 franjas",
            CineStripCount::Four => "4 franjas",
        }
    }
}

/// Per-strip pan/zoom state. `offset_x`/`offset_y` are in output-canvas pixels
/// (same coordinate space as the strip output buffer), not screen pixels.
#[derive(Clone, Copy)]
pub struct PanelOffset {
    pub offset_x: f32,
    pub offset_y: f32,
    /// 1.0 = cover fill; >1 = zoom in.
    pub zoom: f32,
}

impl Default for PanelOffset {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            zoom: 1.0,
        }
    }
}

impl PanelOffset {
    /// Scale factor that makes the image exactly cover a strip of the given size.
    pub fn cover_scale(img_w: f32, img_h: f32, strip_w: f32, strip_h: f32) -> f32 {
        (strip_w / img_w).max(strip_h / img_h)
    }

    pub fn effective_scale(&self, img_w: f32, img_h: f32, strip_w: f32, strip_h: f32) -> f32 {
        Self::cover_scale(img_w, img_h, strip_w, strip_h) * self.zoom
    }

    /// Prevent the image from being panned completely out of the strip.
    pub fn clamp_offset(&mut self, img_w: f32, img_h: f32, strip_w: f32, strip_h: f32) {
        let scale = self.effective_scale(img_w, img_h, strip_w, strip_h);
        let disp_w = img_w * scale;
        let disp_h = img_h * scale;
        let max_x = ((disp_w - strip_w) / 2.0).max(0.0);
        let max_y = ((disp_h - strip_h) / 2.0).max(0.0);
        self.offset_x = self.offset_x.clamp(-max_x, max_x);
        self.offset_y = self.offset_y.clamp(-max_y, max_y);
    }
}

/// Height of strip `i` in output pixels, distributing `canvas_h` among `n` strips.
/// The last strip absorbs any remainder from integer division.
pub fn strip_out_height(canvas_h: u32, n: usize, i: usize) -> u32 {
    let base = canvas_h / n as u32;
    if i == n - 1 {
        canvas_h - base * (n as u32 - 1)
    } else {
        base
    }
}

/// Build default offsets that distribute the source image evenly across strips,
/// each showing a distinct vertical section so the sequence covers the full image.
pub fn default_offsets(
    img_w: u32,
    img_h: u32,
    canvas: CineCanvas,
    count: CineStripCount,
) -> Vec<PanelOffset> {
    let (canvas_w, canvas_h) = canvas.output_size();
    let n = count.count();
    let iw = img_w as f32;
    let ih = img_h as f32;

    (0..n)
        .map(|i| {
            let sh = strip_out_height(canvas_h, n, i) as f32;
            let sw = canvas_w as f32;
            let cover = PanelOffset::cover_scale(iw, ih, sw, sh);

            // Center each strip on a different vertical section of the image.
            // Strip i targets the (i + 0.5)/n fraction of the image height.
            let target_y = (i as f32 + 0.5) / n as f32 * ih;

            // Derive offset_y: with offset_y == 0 the image center (ih/2) sits at
            // the strip center. We need target_y there instead:
            //   ih/2 - offset_y/cover = target_y  →  offset_y = (ih/2 - target_y)*cover
            let offset_y = (ih / 2.0 - target_y) * cover;

            let mut panel = PanelOffset {
                offset_x: 0.0,
                offset_y,
                zoom: 1.0,
            };
            panel.clamp_offset(iw, ih, sw, sh);
            panel
        })
        .collect()
}

/// Draw the interactive strip preview. Returns `true` if any offset was changed
/// (so the caller can invalidate cached render output).
pub fn show_cine_preview(
    ui: &mut egui::Ui,
    texture: &TextureHandle,
    image_dims: (u32, u32),
    canvas: CineCanvas,
    strip_count: CineStripCount,
    offsets: &mut Vec<PanelOffset>,
) -> bool {
    let (out_w, out_h) = canvas.output_size();
    let n = strip_count.count();
    let img_w = image_dims.0 as f32;
    let img_h = image_dims.1 as f32;

    let available_w = ui.available_width().min(600.0);
    let gap = 3.0_f32;
    // Maximum total height for the entire strip stack in the preview panel.
    let max_total_h = 640.0_f32;
    let total_gap = gap * (n as f32 - 1.0);

    // Choose display scale so the full canvas fits both horizontally and vertically.
    let scale_by_w = available_w / out_w as f32;
    let scale_by_h = (max_total_h - total_gap) / out_h as f32;
    let display_scale = scale_by_w.min(scale_by_h).min(1.0);

    let mut changed = false;

    for i in 0..n {
        let strip_out_h_px = strip_out_height(out_h, n, i);
        let strip_disp_w = out_w as f32 * display_scale;
        let strip_disp_h = strip_out_h_px as f32 * display_scale;
        let strip_out_w_f = out_w as f32;
        let strip_out_h_f = strip_out_h_px as f32;

        // Allocate the strip inside a push_id scope so each strip gets a stable, unique ID.
        let alloc = ui.push_id(i, |ui| {
            ui.allocate_exact_size(
                Vec2::new(strip_disp_w, strip_disp_h),
                egui::Sense::click_and_drag(),
            )
        });
        let (strip_rect, response) = alloc.inner;

        if ui.is_rect_visible(strip_rect) {
            let painter = ui.painter();

            // Black background (letterbox colour for cinema feel).
            painter.rect_filled(strip_rect, 0.0, Color32::BLACK);

            let offset = &offsets[i];
            let scale = offset.effective_scale(img_w, img_h, strip_out_w_f, strip_out_h_f);

            // Full scaled-image rect in screen space, centred + offset inside this strip.
            let img_disp_w = img_w * scale * display_scale;
            let img_disp_h = img_h * scale * display_scale;
            let img_center = strip_rect.center()
                + Vec2::new(offset.offset_x, offset.offset_y) * display_scale;
            let img_rect =
                Rect::from_center_size(img_center, Vec2::new(img_disp_w, img_disp_h));

            // Draw the image clipped to this strip.
            painter.with_clip_rect(strip_rect).image(
                texture.id(),
                img_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );

            // Border — highlighted when hovered or dragged.
            let (border_w, border_color) = if response.hovered() || response.dragged() {
                (
                    1.5,
                    Color32::from_rgba_unmultiplied(58, 124, 240, 220),
                )
            } else {
                (1.0, Color32::from_black_alpha(90))
            };
            painter.rect_stroke(
                strip_rect,
                0.0,
                egui::Stroke::new(border_w, border_color),
                egui::StrokeKind::Inside,
            );

            // Small strip-number badge.
            painter.text(
                strip_rect.min + Vec2::new(5.0, 3.0),
                egui::Align2::LEFT_TOP,
                format!("{}", i + 1),
                egui::FontId::proportional(10.0),
                Color32::from_rgba_unmultiplied(255, 255, 255, 170),
            );
        }

        // Drag → pan this strip.
        if response.dragged() {
            let delta = response.drag_delta();
            offsets[i].offset_x += delta.x / display_scale;
            offsets[i].offset_y += delta.y / display_scale;
            offsets[i].clamp_offset(
                img_w,
                img_h,
                out_w as f32,
                strip_out_h_px as f32,
            );
            changed = true;
        }

        // Ctrl / Cmd + scroll → zoom this strip.
        let zoom_mod = ui.input(|inp| inp.modifiers.ctrl || inp.modifiers.command);
        if response.hovered() && zoom_mod {
            let scroll = ui.input(|inp| inp.raw_scroll_delta.y);
            if scroll.abs() > 0.0 {
                let factor = if scroll > 0.0 { 1.1 } else { 1.0 / 1.1 };
                offsets[i].zoom = (offsets[i].zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
                offsets[i].clamp_offset(img_w, img_h, out_w as f32, strip_out_h_px as f32);
                changed = true;
            }
        }

        if i < n - 1 {
            ui.add_space(gap);
        }
    }

    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("Arrastra cada franja para mover · Ctrl + rueda para zoom")
            .weak()
            .size(11.0),
    );

    changed
}

/// Render the final pixel output — one `DynamicImage` per strip at full resolution.
pub fn render_cine(
    image: &DynamicImage,
    canvas: CineCanvas,
    strip_count: CineStripCount,
    offsets: &[PanelOffset],
) -> Vec<DynamicImage> {
    let (out_w, out_h) = canvas.output_size();
    let n = strip_count.count();
    let (img_w, img_h) = image.dimensions();
    let img_wf = img_w as f32;
    let img_hf = img_h as f32;

    let mut results = Vec::with_capacity(n);

    for i in 0..n {
        let sh = strip_out_height(out_h, n, i);
        let sw = out_w as f32;
        let shf = sh as f32;
        let offset = &offsets[i];
        let scale = offset.effective_scale(img_wf, img_hf, sw, shf);

        let scaled_w = (img_wf * scale).round().max(1.0) as u32;
        let scaled_h = (img_hf * scale).round().max(1.0) as u32;

        // Top-left corner of the scaled image inside the strip buffer.
        let paste_x = (sw / 2.0 + offset.offset_x - scaled_w as f32 / 2.0).round() as i64;
        let paste_y = (shf / 2.0 + offset.offset_y - scaled_h as f32 / 2.0).round() as i64;

        let scaled =
            image.resize_exact(scaled_w, scaled_h, image::imageops::FilterType::Lanczos3);
        let mut strip_buf = RgbaImage::from_pixel(out_w, sh, Rgba([0, 0, 0, 255]));
        image::imageops::overlay(&mut strip_buf, &scaled.to_rgba8(), paste_x, paste_y);

        results.push(DynamicImage::ImageRgba8(strip_buf));
    }

    results
}
