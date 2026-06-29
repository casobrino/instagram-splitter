use eframe::egui::{self, epaint::Vertex, Color32, Mesh, Pos2, Rect, Shape, TextureHandle, Vec2};
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

use crate::splitter::OUTPUT_WIDTH;

pub const OUTPUT_HEIGHT_STORY: u32 = 1920;
pub const OUTPUT_HEIGHT_LANDSCAPE: u32 = 566;

pub const PADDING_TOP: f32 = 0.04;
pub const PADDING_SIDE: f32 = 0.04;
pub const DEFAULT_BOTTOM_PADDING: f32 = 0.18;
pub const MIN_BOTTOM_PADDING: f32 = 0.12;
pub const MAX_BOTTOM_PADDING: f32 = 0.25;

pub const MIN_ZOOM: f32 = 0.25;
pub const MAX_ZOOM: f32 = 3.0;
pub const MIN_ROTATION: f32 = -45.0;
pub const MAX_ROTATION: f32 = 45.0;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PolaroidFormat {
    #[default]
    Square,
    Portrait,
    Story,
    Landscape,
}

impl PolaroidFormat {
    pub fn all() -> [PolaroidFormat; 4] {
        [
            PolaroidFormat::Square,
            PolaroidFormat::Portrait,
            PolaroidFormat::Story,
            PolaroidFormat::Landscape,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            PolaroidFormat::Square => "1:1",
            PolaroidFormat::Portrait => "4:5",
            PolaroidFormat::Story => "9:16",
            PolaroidFormat::Landscape => "1.91:1",
        }
    }

    pub fn size_label(self) -> &'static str {
        match self {
            PolaroidFormat::Square => "1080×1080",
            PolaroidFormat::Portrait => "1080×1350",
            PolaroidFormat::Story => "1080×1920",
            PolaroidFormat::Landscape => "1080×566",
        }
    }

    pub fn output_size(self) -> (u32, u32) {
        match self {
            PolaroidFormat::Square => (OUTPUT_WIDTH, OUTPUT_WIDTH),
            PolaroidFormat::Portrait => (OUTPUT_WIDTH, 1350),
            PolaroidFormat::Story => (OUTPUT_WIDTH, OUTPUT_HEIGHT_STORY),
            PolaroidFormat::Landscape => (OUTPUT_WIDTH, OUTPUT_HEIGHT_LANDSCAPE),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum FrameBackground {
    #[default]
    White,
    Cream,
    LightGray,
    Black,
    Custom,
}

impl FrameBackground {
    pub fn presets() -> [FrameBackground; 4] {
        [
            FrameBackground::White,
            FrameBackground::Cream,
            FrameBackground::LightGray,
            FrameBackground::Black,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            FrameBackground::White => "Blanco",
            FrameBackground::Cream => "Crema",
            FrameBackground::LightGray => "Gris",
            FrameBackground::Black => "Negro",
            FrameBackground::Custom => "Personalizado",
        }
    }

    pub fn color(self, custom: Color32) -> Color32 {
        match self {
            FrameBackground::White => Color32::from_rgb(255, 255, 255),
            FrameBackground::Cream => Color32::from_rgb(245, 240, 230),
            FrameBackground::LightGray => Color32::from_rgb(232, 232, 232),
            FrameBackground::Black => Color32::from_rgb(26, 26, 26),
            FrameBackground::Custom => custom,
        }
    }
}

#[derive(Clone, Copy)]
pub struct InnerViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub fn inner_viewport(canvas_w: f32, canvas_h: f32, bottom_padding: f32) -> InnerViewport {
    InnerViewport {
        x: canvas_w * PADDING_SIDE,
        y: canvas_h * PADDING_TOP,
        width: canvas_w * (1.0 - 2.0 * PADDING_SIDE),
        height: canvas_h * (1.0 - PADDING_TOP - bottom_padding),
    }
}

#[derive(Clone, Copy)]
pub struct ImagePlacement {
    /// Desplazamiento desde el centro del viewport, en píxeles del viewport.
    pub offset_x: f32,
    pub offset_y: f32,
    /// 1.0 = cubrir el área; menor = imagen más pequeña; mayor = zoom.
    pub zoom: f32,
    pub rotation_deg: f32,
}

impl Default for ImagePlacement {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            zoom: 1.0,
            rotation_deg: 0.0,
        }
    }
}

impl ImagePlacement {
    pub fn reset() -> Self {
        Self::default()
    }

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
        let angle = self.rotation_deg.to_radians();
        let cos = angle.cos().abs();
        let sin = angle.sin().abs();
        let bound_w = disp_w * cos + disp_h * sin;
        let bound_h = disp_w * sin + disp_h * cos;

        // Imagen más grande que el viewport: pan para recortar.
        // Imagen más pequeña: libertad para moverla por todo el área de foto.
        let max_x = (bound_w - vp_w).abs() / 2.0;
        let max_y = (bound_h - vp_h).abs() / 2.0;

        self.offset_x = self.offset_x.clamp(-max_x, max_x);
        self.offset_y = self.offset_y.clamp(-max_y, max_y);
    }
}

pub fn render_polaroid(
    image: &DynamicImage,
    format: PolaroidFormat,
    bg: Color32,
    bottom_padding: f32,
    placement: &ImagePlacement,
) -> DynamicImage {
    let (out_w, out_h) = format.output_size();
    let canvas_w = out_w as f32;
    let canvas_h = out_h as f32;
    let viewport = inner_viewport(canvas_w, canvas_h, bottom_padding);

    let mut canvas = RgbaImage::from_pixel(
        out_w,
        out_h,
        Rgba([bg.r(), bg.g(), bg.b(), bg.a()]),
    );

    let (img_w, img_h) = image.dimensions();
    let photo = render_photo_in_viewport(
        image,
        img_w as f32,
        img_h as f32,
        viewport.width,
        viewport.height,
        placement,
    );

    image::imageops::overlay(
        &mut canvas,
        &photo,
        viewport.x.round() as i64,
        viewport.y.round() as i64,
    );

    DynamicImage::ImageRgba8(canvas)
}

fn render_photo_in_viewport(
    image: &DynamicImage,
    img_w: f32,
    img_h: f32,
    vp_w: f32,
    vp_h: f32,
    placement: &ImagePlacement,
) -> RgbaImage {
    let inner_w = vp_w.round().max(1.0) as u32;
    let inner_h = vp_h.round().max(1.0) as u32;
    let mut viewport_buf = RgbaImage::from_pixel(inner_w, inner_h, Rgba([0, 0, 0, 0]));

    let (disp_w, disp_h) = placement.display_size(img_w, img_h, vp_w, vp_h);
    let disp_w = disp_w.round().max(1.0) as u32;
    let disp_h = disp_h.round().max(1.0) as u32;

    let scaled = image.resize_exact(disp_w, disp_h, image::imageops::FilterType::Lanczos3);
    let rotated = rotate_rgba(&scaled.to_rgba8(), placement.rotation_deg);

    let paste_x =
        (vp_w / 2.0 + placement.offset_x - rotated.width() as f32 / 2.0).round() as i64;
    let paste_y =
        (vp_h / 2.0 + placement.offset_y - rotated.height() as f32 / 2.0).round() as i64;

    image::imageops::overlay(&mut viewport_buf, &rotated, paste_x, paste_y);
    viewport_buf
}

fn rotate_rgba(image: &RgbaImage, angle_deg: f32) -> RgbaImage {
    if angle_deg.abs() < 0.01 {
        return image.clone();
    }

    let (w, h) = image.dimensions();
    let angle = angle_deg.to_radians();
    let cos = angle.cos();
    let sin = angle.sin();

    let corners = [
        (0.0, 0.0),
        (w as f32, 0.0),
        (w as f32, h as f32),
        (0.0, h as f32),
    ];
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;

    let rotated_corners: Vec<(f32, f32)> = corners
        .iter()
        .map(|(x, y)| {
            let dx = x - cx;
            let dy = y - cy;
            (cos * dx - sin * dy + cx, sin * dx + cos * dy + cy)
        })
        .collect();

    let min_x = rotated_corners
        .iter()
        .map(|(x, _)| x)
        .fold(f32::INFINITY, |a, b| a.min(*b));
    let max_x = rotated_corners
        .iter()
        .map(|(x, _)| x)
        .fold(f32::NEG_INFINITY, |a, b| a.max(*b));
    let min_y = rotated_corners
        .iter()
        .map(|(_, y)| y)
        .fold(f32::INFINITY, |a, b| a.min(*b));
    let max_y = rotated_corners
        .iter()
        .map(|(_, y)| y)
        .fold(f32::NEG_INFINITY, |a, b| a.max(*b));

    let new_w = (max_x - min_x).ceil().max(1.0) as u32;
    let new_h = (max_y - min_y).ceil().max(1.0) as u32;
    let out_cx = new_w as f32 / 2.0;
    let out_cy = new_h as f32 / 2.0;

    let mut out = RgbaImage::from_pixel(new_w, new_h, Rgba([0, 0, 0, 0]));

    for y in 0..new_h {
        for x in 0..new_w {
            let dx = x as f32 - out_cx;
            let dy = y as f32 - out_cy;
            let src_x = cos * dx + sin * dy + cx;
            let src_y = -sin * dx + cos * dy + cy;
            if src_x >= 0.0 && src_y >= 0.0 && src_x < w as f32 - 1.0 && src_y < h as f32 - 1.0 {
                let pixel = sample_bilinear(image, src_x, src_y);
                out.put_pixel(x, y, pixel);
            }
        }
    }

    out
}

fn sample_bilinear(image: &RgbaImage, x: f32, y: f32) -> Rgba<u8> {
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(image.width() - 1);
    let y1 = (y0 + 1).min(image.height() - 1);
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;

    let c00 = image.get_pixel(x0, y0).0;
    let c10 = image.get_pixel(x1, y0).0;
    let c01 = image.get_pixel(x0, y1).0;
    let c11 = image.get_pixel(x1, y1).0;

    let mut out = [0u8; 4];
    for i in 0..4 {
        let v00 = c00[i] as f32;
        let v10 = c10[i] as f32;
        let v01 = c01[i] as f32;
        let v11 = c11[i] as f32;
        let v0 = v00 * (1.0 - tx) + v10 * tx;
        let v1 = v01 * (1.0 - tx) + v11 * tx;
        out[i] = (v0 * (1.0 - ty) + v1 * ty).round().clamp(0.0, 255.0) as u8;
    }
    Rgba(out)
}

pub fn show_polaroid_adjustment_controls(ui: &mut egui::Ui, placement: &mut ImagePlacement) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label("Zoom:");
        if ui
            .add(
                egui::Slider::new(&mut placement.zoom, MIN_ZOOM..=MAX_ZOOM)
                    .logarithmic(true)
                    .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
            )
            .changed()
        {
            changed = true;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Rotación:");
        if ui
            .add(
                egui::Slider::new(&mut placement.rotation_deg, MIN_ROTATION..=MAX_ROTATION)
                    .suffix("°")
                    .fixed_decimals(0),
            )
            .changed()
        {
            changed = true;
        }
    });

    ui.horizontal(|ui| {
        if ui.button("−").on_hover_text("Alejar").clicked() {
            placement.zoom = (placement.zoom / 1.15).clamp(MIN_ZOOM, MAX_ZOOM);
            changed = true;
        }
        if ui.button("＋").on_hover_text("Acercar").clicked() {
            placement.zoom = (placement.zoom * 1.15).clamp(MIN_ZOOM, MAX_ZOOM);
            changed = true;
        }
        if ui.button("↺ 0°").on_hover_text("Quitar rotación").clicked() {
            placement.rotation_deg = 0.0;
            changed = true;
        }
        if ui
            .button("Reiniciar")
            .on_hover_text("Restaurar posición y zoom")
            .clicked()
        {
            *placement = ImagePlacement::reset();
            changed = true;
        }
    });

    changed
}

pub fn show_polaroid_preview(
    ui: &mut egui::Ui,
    texture: &TextureHandle,
    image_dims: (u32, u32),
    format: PolaroidFormat,
    bg: Color32,
    bottom_padding: f32,
    placement: &mut ImagePlacement,
) -> bool {
    let (out_w, out_h) = format.output_size();
    let canvas_w = out_w as f32;
    let canvas_h = out_h as f32;
    let viewport = inner_viewport(canvas_w, canvas_h, bottom_padding);

    let img_w = image_dims.0 as f32;
    let img_h = image_dims.1 as f32;

    let max_display_w = ui.available_width().min(580.0);
    let max_display_h = 680.0_f32;
    let display_scale = (max_display_w / canvas_w)
        .min(max_display_h / canvas_h)
        .min(1.0);
    let display_size = Vec2::new(canvas_w * display_scale, canvas_h * display_scale);

    let mut changed = false;

    let (rect, response) =
        ui.allocate_exact_size(display_size, egui::Sense::click_and_drag());
    let painter = ui.painter();

    if ui.is_rect_visible(rect) {
        painter.rect_filled(rect, 4.0, bg);
        painter.rect_stroke(
            rect,
            4.0,
            (1.0, Color32::from_black_alpha(60)),
            egui::StrokeKind::Outside,
        );

        let vp_display = Rect::from_min_size(
            rect.min
                + Vec2::new(
                    viewport.x * display_scale,
                    viewport.y * display_scale,
                ),
            Vec2::new(
                viewport.width * display_scale,
                viewport.height * display_scale,
            ),
        );

        let vp_center = vp_display.center();
        let (disp_w, disp_h) =
            placement.display_size(img_w, img_h, viewport.width, viewport.height);
        let display_img_size = Vec2::new(
            disp_w * display_scale,
            disp_h * display_scale,
        );

        draw_rotated_image(
            &painter.with_clip_rect(vp_display),
            texture.id(),
            vp_center
                + Vec2::new(placement.offset_x, placement.offset_y) * display_scale,
            display_img_size,
            placement.rotation_deg.to_radians(),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        );

        painter.rect_stroke(
            vp_display,
            0.0,
            (1.0, Color32::from_black_alpha(40)),
            egui::StrokeKind::Inside,
        );
    }

    if response.dragged() {
        let delta = response.drag_delta();
        placement.offset_x += delta.x / display_scale;
        placement.offset_y += delta.y / display_scale;
        placement.clamp_offset(img_w, img_h, viewport.width, viewport.height);
        changed = true;
    }

    let zoom_modifier = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
    if response.hovered() && zoom_modifier {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll.abs() > 0.0 {
            let factor = if scroll > 0.0 { 1.1 } else { 1.0 / 1.1 };
            placement.zoom = (placement.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
            placement.clamp_offset(img_w, img_h, viewport.width, viewport.height);
            changed = true;
        }
    }

    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("Arrastra para mover · Ctrl + rueda para zoom")
            .weak()
            .size(11.0),
    );

    changed
}

fn draw_rotated_image(
    painter: &egui::Painter,
    texture: egui::TextureId,
    center: Pos2,
    size: Vec2,
    angle_rad: f32,
    uv: Rect,
) {
    let half = size / 2.0;
    let corners = [
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
        Vec2::new(half.x, half.y),
        Vec2::new(-half.x, half.y),
    ];
    let uv_corners = [
        Pos2::new(uv.left(), uv.top()),
        Pos2::new(uv.right(), uv.top()),
        Pos2::new(uv.right(), uv.bottom()),
        Pos2::new(uv.left(), uv.bottom()),
    ];

    let rot = egui::emath::Rot2::from_angle(angle_rad);
    let mut mesh = Mesh::with_texture(texture);
    for (corner, uv_pos) in corners.iter().zip(uv_corners.iter()) {
        mesh.vertices.push(Vertex {
            pos: center + rot * *corner,
            uv: *uv_pos,
            color: Color32::WHITE,
        });
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(Shape::mesh(mesh));
}

pub fn show_frame_background_controls(
    ui: &mut egui::Ui,
    background: &mut FrameBackground,
    custom_color: &mut Color32,
) {
    ui.horizontal_wrapped(|ui| {
        for preset in FrameBackground::presets() {
            let color = preset.color(Color32::WHITE);
            let selected = *background == preset;
            let button = egui::Button::new(preset.label())
                .fill(if selected {
                    ui.visuals().selection.bg_fill
                } else {
                    color
                })
                .stroke(if selected {
                    egui::Stroke::new(2.0, ui.visuals().selection.stroke.color)
                } else {
                    egui::Stroke::new(1.0, Color32::from_black_alpha(40))
                })
                .min_size(Vec2::new(72.0, 28.0));
            if ui.add(button).clicked() {
                *background = preset;
            }
        }

        let custom_selected = *background == FrameBackground::Custom;
        if ui
            .selectable_label(custom_selected, "Personalizado")
            .clicked()
        {
            *background = FrameBackground::Custom;
        }
        if *background == FrameBackground::Custom {
            ui.color_edit_button_srgba(custom_color);
        }
    });
}
