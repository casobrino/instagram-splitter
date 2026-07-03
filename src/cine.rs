use eframe::egui::{self, Color32, Pos2, Rect, TextureHandle, Vec2};
use image::{DynamicImage, GenericImageView};

use crate::splitter::OUTPUT_WIDTH;

pub const OUTPUT_HEIGHT_PORTRAIT: u32 = 1350;
pub const OUTPUT_HEIGHT_STORY: u32 = 1920;

pub const MIN_ZOOM: f32 = 0.25;
pub const MAX_ZOOM: f32 = 3.0;

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

    pub fn output_size(self) -> (u32, u32) {
        match self {
            CineCanvas::Portrait45 => (OUTPUT_WIDTH, OUTPUT_HEIGHT_PORTRAIT),
            CineCanvas::Story916 => (OUTPUT_WIDTH, OUTPUT_HEIGHT_STORY),
        }
    }

    pub fn strip_output_size(self, count: CineStripCount) -> (u32, u32) {
        let (w, h) = self.output_size();
        let n = count.as_usize() as u32;
        (w, h / n)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum CineStripCount {
    #[default]
    Three,
    Four,
}

impl CineStripCount {
    pub fn all() -> [CineStripCount; 2] {
        [CineStripCount::Three, CineStripCount::Four]
    }

    pub fn as_usize(self) -> usize {
        match self {
            CineStripCount::Three => 3,
            CineStripCount::Four => 4,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CineStripCount::Three => "3 tiras",
            CineStripCount::Four => "4 tiras",
        }
    }
}

#[derive(Clone, Copy)]
pub struct PanelOffset {
    pub offset_x: f32,
    pub offset_y: f32,
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
    pub fn cover_scale(img_w: f32, img_h: f32, vp_w: f32, vp_h: f32) -> f32 {
        (vp_w / img_w).max(vp_h / img_h)
    }

    pub fn display_scale(&self, img_w: f32, img_h: f32, vp_w: f32, vp_h: f32) -> f32 {
        Self::cover_scale(img_w, img_h, vp_w, vp_h) * self.zoom
    }

    pub fn display_size(&self, img_w: f32, img_h: f32, vp_w: f32, vp_h: f32) -> (f32, f32) {
        let scale = self.display_scale(img_w, img_h, vp_w, vp_h);
        (img_w * scale, img_h * scale)
    }

    pub fn clamp_offset(&mut self, img_w: f32, img_h: f32, vp_w: f32, vp_h: f32) {
        let (disp_w, disp_h) = self.display_size(img_w, img_h, vp_w, vp_h);
        let max_x = (disp_w - vp_w).abs() / 2.0;
        let max_y = (disp_h - vp_h).abs() / 2.0;
        self.offset_x = self.offset_x.clamp(-max_x, max_x);
        self.offset_y = self.offset_y.clamp(-max_y, max_y);
    }
}

pub fn default_offset_for_strip() -> PanelOffset {
    PanelOffset::default()
}

fn source_crop_rect(
    offset: &PanelOffset,
    img_w: f32,
    img_h: f32,
    vp_w: f32,
    vp_h: f32,
) -> (u32, u32, u32, u32) {
    let scale = offset.display_scale(img_w, img_h, vp_w, vp_h);
    let (disp_w, disp_h) = offset.display_size(img_w, img_h, vp_w, vp_h);

    let mut sx = (disp_w / 2.0 - offset.offset_x - vp_w / 2.0) / scale;
    let mut sy = (disp_h / 2.0 - offset.offset_y - vp_h / 2.0) / scale;
    let mut sw = vp_w / scale;
    let mut sh = vp_h / scale;

    if sx < 0.0 {
        sw += sx;
        sx = 0.0;
    }
    if sy < 0.0 {
        sh += sy;
        sy = 0.0;
    }
    if sx + sw > img_w {
        sw = img_w - sx;
    }
    if sy + sh > img_h {
        sh = img_h - sy;
    }

    (
        sx.round().max(0.0) as u32,
        sy.round().max(0.0) as u32,
        sw.round().max(1.0) as u32,
        sh.round().max(1.0) as u32,
    )
}

pub fn render_cine_strip(
    image: &DynamicImage,
    offset: &PanelOffset,
    canvas: CineCanvas,
    strip_count: CineStripCount,
) -> DynamicImage {
    let (img_w, img_h) = image.dimensions();
    let img_w = img_w as f32;
    let img_h = img_h as f32;
    let (out_w, out_h) = canvas.strip_output_size(strip_count);
    let vp_w = out_w as f32;
    let vp_h = out_h as f32;

    let (sx, sy, sw, sh) = source_crop_rect(offset, img_w, img_h, vp_w, vp_h);
    image
        .crop_imm(sx, sy, sw, sh)
        .resize_exact(out_w, out_h, image::imageops::FilterType::Lanczos3)
}

pub fn render_cine(
    images: &[&DynamicImage],
    offsets: &[PanelOffset],
    canvas: CineCanvas,
    strip_count: CineStripCount,
) -> Vec<DynamicImage> {
    images
        .iter()
        .zip(offsets.iter())
        .map(|(image, offset)| render_cine_strip(image, offset, canvas, strip_count))
        .collect()
}

pub struct CineStripView<'a> {
    pub texture: Option<&'a TextureHandle>,
    pub dims: Option<(u32, u32)>,
    pub offset: &'a mut PanelOffset,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum CinePreviewEvent {
    #[default]
    None,
    OffsetChanged,
    StripClicked(usize),
}

pub fn show_cine_preview(
    ui: &mut egui::Ui,
    strips: &mut [CineStripView<'_>],
    canvas: CineCanvas,
    strip_count: CineStripCount,
    selected: Option<usize>,
    accent: Color32,
) -> CinePreviewEvent {
    let (canvas_w, canvas_h) = canvas.output_size();
    let canvas_w = canvas_w as f32;
    let canvas_h = canvas_h as f32;
    let n = strip_count.as_usize();

    let (strip_w, strip_h) = {
        let (sw, sh) = canvas.strip_output_size(strip_count);
        (sw as f32, sh as f32)
    };

    let max_display_w = ui.available_width().min(580.0);
    let max_display_h = 680.0_f32;
    let display_scale = (max_display_w / canvas_w)
        .min(max_display_h / canvas_h)
        .min(1.0);
    let display_size = Vec2::new(canvas_w * display_scale, canvas_h * display_scale);
    let strip_display_h = strip_h * display_scale;

    let mut event = CinePreviewEvent::None;
    let zoom_modifier = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);

    let (outer_rect, _) = ui.allocate_exact_size(display_size, egui::Sense::hover());
    let painter = ui.painter();

    if ui.is_rect_visible(outer_rect) {
        painter.rect_filled(outer_rect, 4.0, Color32::from_gray(18));
        painter.rect_stroke(
            outer_rect,
            4.0,
            (1.0, Color32::from_black_alpha(60)),
            egui::StrokeKind::Outside,
        );
    }

    for i in 0..n {
        let strip_rect = Rect::from_min_size(
            Pos2::new(
                outer_rect.min.x,
                outer_rect.min.y + i as f32 * strip_display_h,
            ),
            Vec2::new(display_size.x, strip_display_h),
        );

        let response =
            ui.interact(strip_rect, ui.id().with(i), egui::Sense::click_and_drag());
        let is_selected = selected == Some(i);
        let has_image = strips[i].texture.is_some() && strips[i].dims.is_some();

        if ui.is_rect_visible(strip_rect) {
            if let (Some(texture), Some(dims)) = (strips[i].texture, strips[i].dims) {
                let img_w = dims.0 as f32;
                let img_h = dims.1 as f32;
                let offset = &*strips[i].offset;
                let (disp_w, disp_h) = offset.display_size(img_w, img_h, strip_w, strip_h);
                let display_img_size = Vec2::new(disp_w * display_scale, disp_h * display_scale);
                let strip_center = strip_rect.center();

                painter.with_clip_rect(strip_rect).image(
                    texture.id(),
                    Rect::from_center_size(
                        strip_center
                            + Vec2::new(offset.offset_x, offset.offset_y) * display_scale,
                        display_img_size,
                    ),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                painter.rect_filled(strip_rect, 0.0, Color32::from_gray(28));
                painter.text(
                    strip_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("Tira {}\nClic para añadir", i + 1),
                    egui::FontId::proportional(12.0),
                    Color32::from_gray(110),
                );
            }

            if i > 0 {
                painter.line_segment(
                    [strip_rect.left_top(), strip_rect.right_top()],
                    (1.0, Color32::from_black_alpha(120)),
                );
            }

            let border_color = if is_selected {
                accent
            } else {
                Color32::from_white_alpha(30)
            };
            let border_w = if is_selected { 2.0 } else { 1.0 };
            painter.rect_stroke(
                strip_rect,
                0.0,
                (border_w, border_color),
                egui::StrokeKind::Inside,
            );

            let label_pos = strip_rect.left_top() + Vec2::new(6.0, 4.0);
            painter.text(
                label_pos,
                egui::Align2::LEFT_TOP,
                format!("{}", i + 1),
                egui::FontId::proportional(11.0),
                Color32::from_white_alpha(180),
            );
        }

        if has_image && response.dragged() {
            let delta = response.drag_delta();
            if let Some(dims) = strips[i].dims {
                let offset = &mut strips[i].offset;
                offset.offset_x += delta.x / display_scale;
                offset.offset_y += delta.y / display_scale;
                offset.clamp_offset(dims.0 as f32, dims.1 as f32, strip_w, strip_h);
            }
            event = CinePreviewEvent::OffsetChanged;
        } else if response.clicked() {
            event = CinePreviewEvent::StripClicked(i);
        }

        if has_image && response.hovered() && zoom_modifier {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll.abs() > 0.0 {
                if let Some(dims) = strips[i].dims {
                    let offset = &mut strips[i].offset;
                    let factor = if scroll > 0.0 { 1.1 } else { 1.0 / 1.1 };
                    offset.zoom = (offset.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
                    offset.clamp_offset(dims.0 as f32, dims.1 as f32, strip_w, strip_h);
                }
                event = CinePreviewEvent::OffsetChanged;
            }
        }
    }

    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(
            "Clic en una tira para añadir o cambiar · Arrastra para mover · Ctrl + rueda para zoom",
        )
        .weak()
        .size(11.0),
    );

    event
}
