use eframe::egui::{self, Color32, Pos2, Rect, TextureHandle, Vec2};
use image::{DynamicImage, GenericImageView};

pub const OUTPUT_WIDTH: u32 = 1080;
pub const OUTPUT_HEIGHT_PORTRAIT: u32 = 1350;
pub const OUTPUT_HEIGHT_SQUARE: u32 = 1080;
const MIN_SLICE_WIDTH: f32 = 100.0;
const MIN_CROP_WIDTH: f32 = 200.0;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitMode {
    TwoSquare,
    #[default]
    TwoPortrait,
    ThreePortrait,
}

impl SplitMode {
    pub fn all() -> [SplitMode; 3] {
        [
            SplitMode::TwoSquare,
            SplitMode::TwoPortrait,
            SplitMode::ThreePortrait,
        ]
    }

    pub fn slice_count(self) -> usize {
        match self {
            SplitMode::TwoSquare | SplitMode::TwoPortrait => 2,
            SplitMode::ThreePortrait => 3,
        }
    }

    pub fn slice_label(self) -> &'static str {
        match self {
            SplitMode::TwoSquare => "1:1",
            SplitMode::TwoPortrait | SplitMode::ThreePortrait => "4:5",
        }
    }

    pub fn crop_aspect(self) -> f32 {
        match self {
            SplitMode::TwoSquare => 2.0,
            SplitMode::TwoPortrait => 8.0 / 5.0,
            SplitMode::ThreePortrait => 12.0 / 5.0,
        }
    }

    pub fn crop_aspect_label(self) -> &'static str {
        match self {
            SplitMode::TwoSquare => "2:1",
            SplitMode::TwoPortrait => "8:5",
            SplitMode::ThreePortrait => "12:5",
        }
    }

    pub fn output_size(self) -> (u32, u32) {
        match self {
            SplitMode::TwoSquare => (OUTPUT_WIDTH, OUTPUT_HEIGHT_SQUARE),
            SplitMode::TwoPortrait | SplitMode::ThreePortrait => {
                (OUTPUT_WIDTH, OUTPUT_HEIGHT_PORTRAIT)
            }
        }
    }

    pub fn min_crop_width(self) -> f32 {
        (self.slice_count() as f32 * MIN_SLICE_WIDTH).max(MIN_CROP_WIDTH)
    }
}

#[derive(Clone, Copy)]
pub struct CropSelection {
    pub width_px: f32,
    pub center_x: f32,
    pub center_y: f32,
}

impl CropSelection {
    pub fn new_centered(img_w: f32, img_h: f32, width_px: f32, mode: SplitMode) -> Self {
        let mut crop = Self {
            width_px,
            center_x: img_w / 2.0,
            center_y: img_h / 2.0,
        };
        crop.clamp_to_image(img_w, img_h, mode);
        crop
    }

    pub fn height_px(&self, mode: SplitMode) -> f32 {
        self.width_px / mode.crop_aspect()
    }

    pub fn rect(&self, mode: SplitMode) -> (f32, f32, f32, f32) {
        let w = self.width_px;
        let h = self.height_px(mode);
        let x = self.center_x - w / 2.0;
        let y = self.center_y - h / 2.0;
        (x, y, w, h)
    }

    pub fn max_width(img_w: f32, img_h: f32, mode: SplitMode) -> f32 {
        let aspect = mode.crop_aspect();
        img_w.min(img_h * aspect).max(mode.min_crop_width())
    }

    pub fn min_width(img_h: f32, mode: SplitMode) -> f32 {
        mode.min_crop_width().min(img_h * mode.crop_aspect())
    }

    pub fn clamp_to_image(&mut self, img_w: f32, img_h: f32, mode: SplitMode) {
        let min_w = Self::min_width(img_h, mode);
        let max_w = Self::max_width(img_w, img_h, mode);
        self.width_px = self.width_px.clamp(min_w, max_w);

        let half_w = self.width_px / 2.0;
        let half_h = self.height_px(mode) / 2.0;
        self.center_x = self.center_x.clamp(half_w, img_w - half_w);
        self.center_y = self.center_y.clamp(half_h, img_h - half_h);
    }
}

/// Muestra solo la imagen con el overlay de recorte (sin slider).
pub fn show_crop_image_only(
    ui: &mut egui::Ui,
    texture: &TextureHandle,
    image_dims: (u32, u32),
    crop: &mut CropSelection,
    mode: SplitMode,
) {
    let img_w = image_dims.0 as f32;
    let img_h = image_dims.1 as f32;
    let max_display_width = ui.available_width().max(100.0);
    let max_display_height = (ui.available_height() - 8.0).max(100.0);
    let slice_count = mode.slice_count();

    let size = texture.size_vec2();
    let scale = (max_display_width / size.x)
        .min(max_display_height / size.y)
        .min(1.0);
    let display_size = size * scale;

    let (rect, response) =
        ui.allocate_exact_size(display_size, egui::Sense::click_and_drag());
    let painter = ui.painter();

    if ui.is_rect_visible(rect) {
        painter.image(
            texture.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        let crop_display = image_rect_to_display(crop.rect(mode), rect, image_dims);
        draw_overlay_outside_crop(&painter, rect, crop_display);
        draw_dashed_rect(&painter, crop_display, Color32::WHITE, 2.0);

        for i in 1..slice_count {
            let t = i as f32 / slice_count as f32;
            let div_x = crop_display.left() + crop_display.width() * t;
            draw_dashed_line(
                &painter,
                Pos2::new(div_x, crop_display.top()),
                Pos2::new(div_x, crop_display.bottom()),
                Color32::WHITE,
                2.0,
            );
        }

        for i in 0..slice_count {
            let t0 = i as f32 / slice_count as f32;
            let t1 = (i + 1) as f32 / slice_count as f32;
            let center_x =
                crop_display.left() + crop_display.width() * (t0 + t1) / 2.0;
            painter.text(
                Pos2::new(center_x, crop_display.center().y),
                egui::Align2::CENTER_CENTER,
                format!("Foto {}", i + 1),
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );
        }
    }

    if response.dragged() {
        let scale_x = img_w / rect.width();
        let scale_y = img_h / rect.height();
        let delta = response.drag_delta();
        crop.center_x += delta.x * scale_x;
        crop.center_y += delta.y * scale_y;
        crop.clamp_to_image(img_w, img_h, mode);
    }
}

/// Muestra el slider del ancho de recorte y las etiquetas de info.
pub fn show_crop_slider(
    ui: &mut egui::Ui,
    image_dims: (u32, u32),
    crop: &mut CropSelection,
    mode: SplitMode,
) {
    let img_w = image_dims.0 as f32;
    let img_h = image_dims.1 as f32;
    let min_w = CropSelection::min_width(img_h, mode);
    let max_w = CropSelection::max_width(img_w, img_h, mode);
    if ui
        .add(
            egui::Slider::new(&mut crop.width_px, min_w..=max_w)
                .suffix(" px")
                .fixed_decimals(0),
        )
        .changed()
    {
        crop.clamp_to_image(img_w, img_h, mode);
    }
}

pub fn show_texture_preview(ui: &mut egui::Ui, texture: &TextureHandle, max_width: f32) {
    let size = texture.size_vec2();
    let scale = (max_width / size.x).min(1.0);
    let display_size = size * scale;
    ui.add(
        egui::Image::new(texture)
            .fit_to_exact_size(display_size)
            .corner_radius(4.0),
    );
}

pub fn split_crop_region(
    image: &DynamicImage,
    crop: &CropSelection,
    mode: SplitMode,
) -> Result<Vec<DynamicImage>, String> {
    let (img_w, img_h) = image.dimensions();

    if img_w <= img_h {
        return Err(
            "La imagen debe estar en formato horizontal (acostada).".to_string(),
        );
    }

    let (x, y, w, h) = crop.rect(mode);
    let x = x.round() as u32;
    let y = y.round() as u32;
    let w = w.round() as u32;
    let h = h.round() as u32;

    if w < mode.min_crop_width() as u32 {
        return Err("El recorte es demasiado pequeño. Aumenta el ancho.".to_string());
    }

    if x + w > img_w || y + h > img_h {
        return Err("El recorte sale de los límites de la imagen.".to_string());
    }

    let slice_count = mode.slice_count();
    let base_slice_w = w / slice_count as u32;
    if base_slice_w == 0 {
        return Err(format!(
            "El recorte es demasiado pequeño para dividir en {} fotos.",
            slice_count
        ));
    }

    let (out_w, out_h) = mode.output_size();
    let filter = image::imageops::FilterType::Lanczos3;
    let mut outputs = Vec::with_capacity(slice_count);

    for i in 0..slice_count {
        let offset_x = x + base_slice_w * i as u32;
        let slice_w = if i == slice_count - 1 {
            w - base_slice_w * (slice_count as u32 - 1)
        } else {
            base_slice_w
        };

        let piece = image
            .crop_imm(offset_x, y, slice_w, h)
            .resize_exact(out_w, out_h, filter);
        outputs.push(piece);
    }

    Ok(outputs)
}

fn image_rect_to_display(
    image_rect: (f32, f32, f32, f32),
    display_rect: Rect,
    image_dims: (u32, u32),
) -> Rect {
    let scale_x = display_rect.width() / image_dims.0 as f32;
    let scale_y = display_rect.height() / image_dims.1 as f32;
    let (ix, iy, iw, ih) = image_rect;
    Rect::from_min_size(
        display_rect.min + Vec2::new(ix * scale_x, iy * scale_y),
        Vec2::new(iw * scale_x, ih * scale_y),
    )
}

fn draw_overlay_outside_crop(painter: &egui::Painter, image_rect: Rect, crop_rect: Rect) {
    let overlay = Color32::from_black_alpha(120);

    if crop_rect.top() > image_rect.top() {
        painter.rect_filled(
            Rect::from_min_max(
                image_rect.left_top(),
                Pos2::new(image_rect.right(), crop_rect.top()),
            ),
            0.0,
            overlay,
        );
    }
    if crop_rect.bottom() < image_rect.bottom() {
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(image_rect.left(), crop_rect.bottom()),
                image_rect.right_bottom(),
            ),
            0.0,
            overlay,
        );
    }
    if crop_rect.left() > image_rect.left() {
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(image_rect.left(), crop_rect.top()),
                Pos2::new(crop_rect.left(), crop_rect.bottom()),
            ),
            0.0,
            overlay,
        );
    }
    if crop_rect.right() < image_rect.right() {
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(crop_rect.right(), crop_rect.top()),
                Pos2::new(image_rect.right(), crop_rect.bottom()),
            ),
            0.0,
            overlay,
        );
    }
}

fn draw_dashed_line(
    painter: &egui::Painter,
    start: Pos2,
    end: Pos2,
    color: Color32,
    width: f32,
) {
    const DASH: f32 = 6.0;
    const GAP: f32 = 4.0;

    let dir = end - start;
    let len = dir.length();
    if len < 0.001 {
        return;
    }
    let dir = dir / len;
    let mut pos = 0.0;
    while pos < len {
        let seg_end = (pos + DASH).min(len);
        painter.line_segment(
            [start + dir * pos, start + dir * seg_end],
            (width, color),
        );
        pos += DASH + GAP;
    }
}

fn draw_dashed_rect(painter: &egui::Painter, rect: Rect, color: Color32, width: f32) {
    draw_dashed_line(
        painter,
        rect.left_top(),
        rect.right_top(),
        color,
        width,
    );
    draw_dashed_line(
        painter,
        rect.right_top(),
        rect.right_bottom(),
        color,
        width,
    );
    draw_dashed_line(
        painter,
        rect.right_bottom(),
        rect.left_bottom(),
        color,
        width,
    );
    draw_dashed_line(
        painter,
        rect.left_bottom(),
        rect.left_top(),
        color,
        width,
    );
}
