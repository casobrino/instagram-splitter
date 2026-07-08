mod cine;
mod polaroid;
mod splitter;

use eframe::egui::{self, Color32, Pos2, Rect, TextureHandle, Vec2};
use image::{DynamicImage, GenericImageView, ImageFormat};

use cine::{
    show_cine_preview, CineCanvas, CinePreviewEvent, CineStripCount, CineStripView, PanelOffset,
};
use polaroid::{
    show_frame_background_controls, show_polaroid_adjustment_controls, show_polaroid_preview,
    FrameBackground, ImagePlacement, PolaroidFormat, DEFAULT_BOTTOM_PADDING, MAX_BOTTOM_PADDING,
    MIN_BOTTOM_PADDING,
};
use splitter::{
    show_crop_image_only, show_crop_slider, show_texture_preview, split_crop_region,
    CropSelection, SplitMode,
};

const SIDEBAR_W: f32 = 272.0;
const ACCENT: Color32 = Color32::from_rgb(58, 124, 240);
const TEAL: Color32 = Color32::from_rgb(48, 168, 120);
const PANEL_BG: Color32 = Color32::from_gray(26);
const CARD_BG: Color32 = Color32::from_gray(44);
const PREVIEW_BG: Color32 = Color32::from_gray(34);

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 760.0])
            .with_title("Instagram Photo Splitter")
            .with_min_inner_size([780.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Instagram Photo Splitter",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum AppTab {
    #[default]
    Splitter,
    Polaroid,
    Cine,
}

struct CineStripSlot {
    image: Option<DynamicImage>,
    texture: Option<TextureHandle>,
    offset: PanelOffset,
}

impl Default for CineStripSlot {
    fn default() -> Self {
        Self {
            image: None,
            texture: None,
            offset: PanelOffset::default(),
        }
    }
}

struct App {
    active_tab: AppTab,
    status: String,

    original_image: Option<DynamicImage>,
    original_texture: Option<TextureHandle>,
    crop_selection: Option<CropSelection>,
    split_mode: SplitMode,
    output_images: Option<Vec<DynamicImage>>,
    output_textures: Option<Vec<TextureHandle>>,

    polaroid_image: Option<DynamicImage>,
    polaroid_texture: Option<TextureHandle>,
    polaroid_placement: Option<ImagePlacement>,
    polaroid_format: PolaroidFormat,
    polaroid_background: FrameBackground,
    polaroid_custom_color: egui::Color32,
    polaroid_bottom_padding: f32,
    polaroid_output: Option<DynamicImage>,
    polaroid_output_texture: Option<TextureHandle>,

    cine_canvas: CineCanvas,
    cine_strip_count: CineStripCount,
    cine_slots: Vec<CineStripSlot>,
    cine_selected_strip: Option<usize>,
    cine_output: Option<DynamicImage>,
    cine_output_texture: Option<TextureHandle>,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            active_tab: AppTab::default(),
            status: String::new(),
            original_image: None,
            original_texture: None,
            crop_selection: None,
            split_mode: SplitMode::default(),
            output_images: None,
            output_textures: None,
            polaroid_image: None,
            polaroid_texture: None,
            polaroid_placement: None,
            polaroid_format: PolaroidFormat::default(),
            polaroid_background: FrameBackground::default(),
            polaroid_custom_color: egui::Color32::from_rgb(200, 180, 160),
            polaroid_bottom_padding: DEFAULT_BOTTOM_PADDING,
            polaroid_output: None,
            polaroid_output_texture: None,
            cine_canvas: CineCanvas::default(),
            cine_strip_count: CineStripCount::default(),
            cine_slots: Vec::new(),
            cine_selected_strip: None,
            cine_output: None,
            cine_output_texture: None,
        };
        app.ensure_cine_slots();
        app
    }
}

impl App {
    fn init_crop_for_current_image(&mut self) {
        let Some(image) = self.original_image.as_ref() else {
            return;
        };
        let (img_w, img_h) = image.dimensions();
        let max_w = CropSelection::max_width(img_w as f32, img_h as f32, self.split_mode);
        self.crop_selection = Some(CropSelection::new_centered(
            img_w as f32,
            img_h as f32,
            max_w,
            self.split_mode,
        ));
        self.output_images = None;
        self.output_textures = None;
    }

    fn init_polaroid_placement(&mut self) {
        if self.polaroid_image.is_some() {
            self.polaroid_placement = Some(ImagePlacement::reset());
            self.polaroid_output = None;
            self.polaroid_output_texture = None;
        }
    }

    fn invalidate_polaroid_output(&mut self) {
        self.polaroid_output = None;
        self.polaroid_output_texture = None;
    }

    fn ensure_cine_slots(&mut self) {
        let n = self.cine_strip_count.as_usize();
        self.cine_slots.resize_with(n, CineStripSlot::default);
        if self.cine_selected_strip.map(|i| i >= n) == Some(true) {
            self.cine_selected_strip = None;
        }
    }

    fn cine_filled_count(&self) -> usize {
        self.cine_slots.iter().filter(|s| s.image.is_some()).count()
    }

    fn cine_all_slots_filled(&self) -> bool {
        let n = self.cine_strip_count.as_usize();
        self.cine_slots.len() == n && self.cine_slots.iter().all(|s| s.image.is_some())
    }

    fn next_empty_cine_slot(&self) -> Option<usize> {
        self.cine_slots.iter().position(|s| s.image.is_none())
    }

    fn invalidate_cine_output(&mut self) {
        self.cine_output = None;
        self.cine_output_texture = None;
    }

    fn set_cine_strip_image(&mut self, ctx: &egui::Context, idx: usize, img: DynamicImage) {
        if idx >= self.cine_slots.len() {
            return;
        }
        let slot = &mut self.cine_slots[idx];
        slot.texture = Some(dynamic_image_to_texture(
            ctx,
            &format!("cine_strip_{idx}"),
            &img,
        ));
        slot.offset = cine::default_offset_for_strip();
        slot.image = Some(img);
        self.cine_selected_strip = Some(idx);
        self.invalidate_cine_output();
    }

    fn pick_cine_image_for_slot(&mut self, ctx: &egui::Context, idx: usize) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Imágenes", &["jpg", "jpeg", "png"])
            .pick_file()
        {
            match image::open(&path) {
                Ok(img) => {
                    self.set_cine_strip_image(ctx, idx, img);
                    self.status = format!(
                        "Tira {}: {}",
                        idx + 1,
                        path.file_name().unwrap_or_default().to_string_lossy()
                    );
                }
                Err(err) => {
                    self.status = format!("Error al abrir la imagen: {err}");
                }
            }
        }
    }

    fn open_image(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Imágenes", &["jpg", "jpeg", "png"])
            .pick_file()
        {
            match image::open(&path) {
                Ok(img) => {
                    let (img_w, img_h) = img.dimensions();
                    if img_w <= img_h {
                        self.original_image = None;
                        self.original_texture = None;
                        self.crop_selection = None;
                        self.output_images = None;
                        self.output_textures = None;
                        self.status =
                            "La imagen debe estar en formato horizontal (acostada).".to_string();
                        return;
                    }
                    self.original_texture =
                        Some(dynamic_image_to_texture(ctx, "original", &img));
                    self.original_image = Some(img);
                    self.init_crop_for_current_image();
                    self.status = format!(
                        "{}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    );
                }
                Err(err) => {
                    self.status = format!("Error al abrir la imagen: {err}");
                }
            }
        }
    }

    fn open_polaroid_image(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Imágenes", &["jpg", "jpeg", "png"])
            .pick_file()
        {
            match image::open(&path) {
                Ok(img) => {
                    self.polaroid_texture =
                        Some(dynamic_image_to_texture(ctx, "polaroid_src", &img));
                    self.polaroid_image = Some(img);
                    self.init_polaroid_placement();
                    self.status = format!(
                        "{}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    );
                }
                Err(err) => {
                    self.status = format!("Error al abrir la imagen: {err}");
                }
            }
        }
    }

    fn open_cine_image(&mut self, ctx: &egui::Context) {
        self.ensure_cine_slots();
        let idx = self
            .cine_selected_strip
            .or_else(|| self.next_empty_cine_slot())
            .unwrap_or(0);
        self.pick_cine_image_for_slot(ctx, idx);
    }

    fn convert_image(&mut self, ctx: &egui::Context) {
        let Some(original) = self.original_image.as_ref() else {
            self.status = "Primero abre una imagen".to_string();
            return;
        };
        let Some(crop) = self.crop_selection else {
            self.status = "No hay un recorte válido seleccionado".to_string();
            return;
        };
        match split_crop_region(original, &crop, self.split_mode) {
            Ok(outputs) => {
                let textures = outputs
                    .iter()
                    .enumerate()
                    .map(|(i, img)| {
                        dynamic_image_to_texture(ctx, &format!("output_{}", i + 1), img)
                    })
                    .collect();
                self.output_images = Some(outputs);
                self.output_textures = Some(textures);
                self.status = format!(
                    "{} fotos generadas. Pulsa Guardar.",
                    self.split_mode.slice_count()
                );
            }
            Err(err) => {
                self.output_images = None;
                self.output_textures = None;
                self.status = err;
            }
        }
    }

    fn generate_polaroid(&mut self, ctx: &egui::Context) {
        let Some(image) = self.polaroid_image.as_ref() else {
            self.status = "Primero abre una imagen".to_string();
            return;
        };
        let Some(placement) = self.polaroid_placement else {
            self.status = "No hay una posición válida para la imagen".to_string();
            return;
        };
        let bg = self.polaroid_background.color(self.polaroid_custom_color);
        let output = polaroid::render_polaroid(
            image,
            self.polaroid_format,
            bg,
            self.polaroid_bottom_padding,
            &placement,
        );
        self.polaroid_output_texture =
            Some(dynamic_image_to_texture(ctx, "polaroid_output", &output));
        self.polaroid_output = Some(output);
        let (w, h) = self.polaroid_format.output_size();
        self.status = format!("Polaroid listo ({w}×{h} px).");
    }

    fn generate_cine(&mut self, ctx: &egui::Context) {
        if !self.cine_all_slots_filled() {
            self.status = "Añade una imagen a cada tira antes de generar".to_string();
            return;
        }
        let images: Vec<&DynamicImage> = self
            .cine_slots
            .iter()
            .map(|s| s.image.as_ref().unwrap())
            .collect();
        let offsets: Vec<PanelOffset> = self.cine_slots.iter().map(|s| s.offset).collect();
        let output = cine::render_cine(
            &images,
            &offsets,
            self.cine_canvas,
            self.cine_strip_count,
        );
        let texture = dynamic_image_to_texture(ctx, "cine_output", &output);
        let (w, h) = self.cine_canvas.output_size();
        self.cine_output = Some(output);
        self.cine_output_texture = Some(texture);
        self.status = format!("Imagen cine lista ({w}×{h} px).");
    }

    fn save_images(&mut self) {
        let Some(outputs) = self.output_images.as_ref() else {
            self.status = "Primero convierte una imagen".to_string();
            return;
        };
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            let mut errors = Vec::new();
            for (i, img) in outputs.iter().enumerate() {
                let path = folder.join(format!("foto_{}.jpg", i + 1));
                if let Err(err) = img.save_with_format(&path, ImageFormat::Jpeg) {
                    errors.push(format!("foto_{}: {err}", i + 1));
                }
            }
            if errors.is_empty() {
                self.status = format!("Guardado en {}", folder.to_string_lossy());
            } else {
                self.status = format!("Error: {}", errors.join("; "));
            }
        }
    }

    fn save_polaroid(&mut self) {
        let Some(output) = self.polaroid_output.as_ref() else {
            self.status = "Primero genera el polaroid".to_string();
            return;
        };
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JPEG", &["jpg", "jpeg"])
            .set_file_name("polaroid.jpg")
            .save_file()
        {
            match output.save_with_format(&path, ImageFormat::Jpeg) {
                Ok(()) => {
                    self.status = format!("Guardado en {}", path.to_string_lossy());
                }
                Err(err) => {
                    self.status = format!("Error: {err}");
                }
            }
        }
    }

    fn save_cine_images(&mut self) {
        let Some(output) = self.cine_output.as_ref() else {
            self.status = "Primero genera la imagen".to_string();
            return;
        };
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JPEG", &["jpg", "jpeg"])
            .set_file_name("cine.jpg")
            .save_file()
        {
            match output.save_with_format(&path, ImageFormat::Jpeg) {
                Ok(()) => {
                    self.status = format!("Guardado en {}", path.to_string_lossy());
                }
                Err(err) => {
                    self.status = format!("Error: {err}");
                }
            }
        }
    }

    // ─── Sidebar ────────────────────────────────────────────────────────────

    fn show_sidebar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let full_h = ui.available_height();
        let btn_h = 52.0;
        let content_max_h = (full_h - btn_h - 12.0).max(80.0);

        egui::ScrollArea::vertical()
            .max_height(content_max_h)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // App title
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Instagram Photo Splitter")
                        .strong()
                        .size(14.0),
                );
                ui.add_space(12.0);

                // Mode toggle
                show_mode_toggle(ui, &mut self.active_tab);
                ui.add_space(16.0);

                match self.active_tab {
                    AppTab::Splitter => self.show_splitter_sidebar_content(ui),
                    AppTab::Polaroid => self.show_polaroid_sidebar_content(ui),
                    AppTab::Cine => self.show_cine_sidebar_content(ui),
                }
            });

        // Fill remaining space to push buttons to bottom
        let remaining = ui.available_height() - btn_h - 8.0;
        if remaining > 0.0 {
            ui.add_space(remaining);
        }
        ui.separator();
        ui.add_space(6.0);
        self.show_action_buttons(ui, ctx);
    }

    fn show_splitter_sidebar_content(&mut self, ui: &mut egui::Ui) {
        sidebar_label(ui, "Modo de recorte");
        ui.add_space(6.0);
        self.show_split_mode_cards(ui);

        if self.original_image.is_some() {
            ui.add_space(14.0);
            sidebar_label(ui, "Ancho del recorte");
            ui.add_space(4.0);
            let img_dims = self.original_image.as_ref().map(|i| i.dimensions());
            let mode = self.split_mode;
            if let (Some(dims), Some(crop)) =
                (img_dims, self.crop_selection.as_mut())
            {
                show_crop_slider(ui, dims, crop, mode);
            }
        }
    }

    fn show_polaroid_sidebar_content(&mut self, ui: &mut egui::Ui) {
        sidebar_label(ui, "Formato de salida");
        ui.add_space(6.0);
        self.show_polaroid_format_cards(ui);

        if self.polaroid_image.is_some() {
            ui.add_space(14.0);
            sidebar_label(ui, "Fondo del marco");
            ui.add_space(6.0);
            show_frame_background_controls(
                ui,
                &mut self.polaroid_background,
                &mut self.polaroid_custom_color,
            );

            ui.add_space(12.0);
            sidebar_label(ui, "Borde inferior");
            ui.add_space(4.0);
            if ui
                .add(
                    egui::Slider::new(
                        &mut self.polaroid_bottom_padding,
                        MIN_BOTTOM_PADDING..=MAX_BOTTOM_PADDING,
                    )
                    .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                )
                .changed()
            {
                self.invalidate_polaroid_output();
            }

            let img_dims = self.polaroid_image.as_ref().map(|i| i.dimensions());
            let out_dims = self.polaroid_format.output_size();
            let bottom_pad = self.polaroid_bottom_padding;

            if self.polaroid_placement.is_some() {
                ui.add_space(12.0);
                sidebar_label(ui, "Ajuste de foto");
                ui.add_space(4.0);
            }

            let placement_changed =
                if let Some(placement) = self.polaroid_placement.as_mut() {
                    show_polaroid_adjustment_controls(ui, placement)
                } else {
                    false
                };

            if placement_changed {
                if let (Some(dims), Some(placement)) =
                    (img_dims, self.polaroid_placement.as_mut())
                {
                    let viewport = polaroid::inner_viewport(
                        out_dims.0 as f32,
                        out_dims.1 as f32,
                        bottom_pad,
                    );
                    placement.clamp_offset(
                        dims.0 as f32,
                        dims.1 as f32,
                        viewport.width,
                        viewport.height,
                    );
                }
                self.invalidate_polaroid_output();
            }
        }
    }

    fn show_cine_sidebar_content(&mut self, ui: &mut egui::Ui) {
        sidebar_label(ui, "Lienzo de salida");
        ui.add_space(6.0);
        self.show_cine_canvas_cards(ui);

        ui.add_space(14.0);
        sidebar_label(ui, "Número de tiras");
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for count in CineStripCount::all() {
                let selected = self.cine_strip_count == count;
                let btn = egui::Button::new(count.label())
                    .min_size(Vec2::new(100.0, 32.0))
                    .fill(if selected {
                        TEAL
                    } else {
                        CARD_BG
                    });
                if ui.add(btn).clicked() {
                    self.cine_strip_count = count;
                    self.ensure_cine_slots();
                    self.invalidate_cine_output();
                }
            }
        });

        ui.add_space(12.0);
        let filled = self.cine_filled_count();
        let total = self.cine_strip_count.as_usize();
        sidebar_label(ui, &format!("Imágenes ({filled}/{total})"));
        ui.add_space(4.0);
        for i in 0..total {
            let has_image = self.cine_slots.get(i).is_some_and(|s| s.image.is_some());
            let is_selected = self.cine_selected_strip == Some(i);
            let label = if has_image {
                format!("Tira {} — imagen cargada", i + 1)
            } else {
                format!("Tira {} — vacía", i + 1)
            };
            if ui.selectable_label(is_selected, label).clicked() {
                self.cine_selected_strip = Some(i);
            }
        }
    }

    fn show_action_buttons(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            match self.active_tab {
                AppTab::Splitter => {
                    if ui
                        .add(
                            egui::Button::new("Abrir imagen")
                                .min_size(Vec2::new(80.0, 32.0)),
                        )
                        .clicked()
                    {
                        self.open_image(ctx);
                    }
                    let can_convert =
                        self.original_image.is_some() && self.crop_selection.is_some();
                    let btn = egui::Button::new("Convertir")
                        .fill(if can_convert {
                            ACCENT
                        } else {
                            Color32::from_gray(48)
                        })
                        .min_size(Vec2::new(78.0, 32.0));
                    if ui.add_enabled(can_convert, btn).clicked() {
                        self.convert_image(ctx);
                    }
                    let can_save = self.output_images.is_some();
                    if ui
                        .add_enabled(
                            can_save,
                            egui::Button::new("Guardar").min_size(Vec2::new(62.0, 32.0)),
                        )
                        .clicked()
                    {
                        self.save_images();
                    }
                }
                AppTab::Polaroid => {
                    if ui
                        .add(
                            egui::Button::new("Abrir imagen")
                                .min_size(Vec2::new(80.0, 32.0)),
                        )
                        .clicked()
                    {
                        self.open_polaroid_image(ctx);
                    }
                    let can_gen = self.polaroid_image.is_some()
                        && self.polaroid_placement.is_some();
                    let btn = egui::Button::new("Generar")
                        .fill(if can_gen {
                            ACCENT
                        } else {
                            Color32::from_gray(48)
                        })
                        .min_size(Vec2::new(78.0, 32.0));
                    if ui.add_enabled(can_gen, btn).clicked() {
                        self.generate_polaroid(ctx);
                    }
                    let can_save = self.polaroid_output.is_some();
                    if ui
                        .add_enabled(
                            can_save,
                            egui::Button::new("Guardar").min_size(Vec2::new(62.0, 32.0)),
                        )
                        .clicked()
                    {
                        self.save_polaroid();
                    }
                }
                AppTab::Cine => {
                    let filled = self.cine_filled_count();
                    let total = self.cine_strip_count.as_usize();
                    let btn_label = if filled < total {
                        "Añadir imagen"
                    } else {
                        "Cambiar imagen"
                    };
                    if ui
                        .add(
                            egui::Button::new(btn_label).min_size(Vec2::new(100.0, 32.0)),
                        )
                        .clicked()
                    {
                        self.open_cine_image(ctx);
                    }
                    let can_gen = self.cine_all_slots_filled();
                    let btn = egui::Button::new("Generar")
                        .fill(if can_gen {
                            ACCENT
                        } else {
                            Color32::from_gray(48)
                        })
                        .min_size(Vec2::new(78.0, 32.0));
                    if ui.add_enabled(can_gen, btn).clicked() {
                        self.generate_cine(ctx);
                    }
                    let can_save = self.cine_output.is_some();
                    if ui
                        .add_enabled(
                            can_save,
                            egui::Button::new("Guardar").min_size(Vec2::new(62.0, 32.0)),
                        )
                        .clicked()
                    {
                        self.save_cine_images();
                    }
                }
            }
        });
    }

    // ─── Format cards ───────────────────────────────────────────────────────

    fn show_split_mode_cards(&mut self, ui: &mut egui::Ui) {
        let card_w = ((SIDEBAR_W - 32.0 - 8.0) / 2.0).floor();
        let card_h = 126.0_f32;

        // two per row
        let modes = SplitMode::all();
        let row1 = &modes[..2];
        let row2 = &modes[2..];

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for &mode in row1 {
                let selected = self.split_mode == mode;
                if draw_split_mode_card(ui, mode, selected, Vec2::new(card_w, card_h)) {
                    self.split_mode = mode;
                    if self.original_image.is_some() {
                        self.init_crop_for_current_image();
                    }
                }
            }
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for &mode in row2 {
                let selected = self.split_mode == mode;
                if draw_split_mode_card(ui, mode, selected, Vec2::new(card_w, card_h)) {
                    self.split_mode = mode;
                    if self.original_image.is_some() {
                        self.init_crop_for_current_image();
                    }
                }
            }
        });
    }

    fn show_polaroid_format_cards(&mut self, ui: &mut egui::Ui) {
        let card_w = ((SIDEBAR_W - 32.0 - 8.0) / 2.0).floor();
        let card_h = 118.0_f32;
        let formats = PolaroidFormat::all();

        for row in formats.chunks(2) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                for &fmt in row {
                    let selected = self.polaroid_format == fmt;
                    if draw_polaroid_format_card(
                        ui,
                        fmt,
                        selected,
                        Vec2::new(card_w, card_h),
                    ) {
                        self.polaroid_format = fmt;
                        if self.polaroid_image.is_some() {
                            self.init_polaroid_placement();
                        }
                    }
                }
            });
            ui.add_space(8.0);
        }
    }

    fn show_cine_canvas_cards(&mut self, ui: &mut egui::Ui) {
        let card_w = ((SIDEBAR_W - 32.0 - 8.0) / 2.0).floor();
        let card_h = 118.0_f32;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for canvas in CineCanvas::all() {
                let selected = self.cine_canvas == canvas;
                if draw_cine_canvas_card(ui, canvas, selected, Vec2::new(card_w, card_h)) {
                    self.cine_canvas = canvas;
                    self.invalidate_cine_output();
                }
            }
        });
    }

    // ─── Preview panel ──────────────────────────────────────────────────────

    fn show_preview(&mut self, ui: &mut egui::Ui) {
        match self.active_tab {
            AppTab::Splitter => self.show_splitter_preview(ui),
            AppTab::Polaroid => self.show_polaroid_preview_panel(ui),
            AppTab::Cine => self.show_cine_preview_panel(ui),
        }
    }

    fn show_splitter_preview(&mut self, ui: &mut egui::Ui) {
        let mode = self.split_mode;

        if let Some(textures) = self.output_textures.as_ref() {
            let count = textures.len();
            let max_w = if count == 3 { 200.0 } else { 280.0 };
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(format!("{count} fotos generadas — lista para guardar"))
                        .strong(),
                );
                ui.add_space(16.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    for (i, texture) in textures.iter().enumerate() {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(format!("Foto {}", i + 1)).weak(),
                            );
                            show_texture_preview(ui, texture, max_w);
                        });
                    }
                });
            });
        } else if let (Some(texture), Some(image), Some(crop)) = (
            self.original_texture.as_ref(),
            self.original_image.as_ref(),
            self.crop_selection.as_mut(),
        ) {
            show_crop_image_only(ui, texture, image.dimensions(), crop, mode);
        } else {
            show_empty_state(ui, "Abre una imagen horizontal para empezar");
        }
    }

    fn show_polaroid_preview_panel(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let bg = self.polaroid_background.color(self.polaroid_custom_color);
                let img_dims = self.polaroid_image.as_ref().map(|i| i.dimensions());

                if let (Some(texture), Some(dims), Some(placement)) = (
                    self.polaroid_texture.as_ref(),
                    img_dims,
                    self.polaroid_placement.as_mut(),
                ) {
                    if show_polaroid_preview(
                        ui,
                        texture,
                        dims,
                        self.polaroid_format,
                        bg,
                        self.polaroid_bottom_padding,
                        placement,
                    ) {
                        self.polaroid_output = None;
                        self.polaroid_output_texture = None;
                    }
                    if let Some(out_tex) = &self.polaroid_output_texture {
                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("Resultado generado").strong());
                        ui.add_space(8.0);
                        show_texture_preview(ui, out_tex, 400.0);
                    }
                } else {
                    show_empty_state(ui, "Abre una imagen para empezar");
                }
            });
    }

    fn show_cine_preview_panel(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.ensure_cine_slots();
                let n = self.cine_strip_count.as_usize();

                let mut views: Vec<CineStripView<'_>> = self
                    .cine_slots
                    .iter_mut()
                    .take(n)
                    .map(|slot| CineStripView {
                        texture: slot.texture.as_ref(),
                        dims: slot.image.as_ref().map(|img| img.dimensions()),
                        offset: &mut slot.offset,
                    })
                    .collect();

                match show_cine_preview(
                    ui,
                    &mut views,
                    self.cine_canvas,
                    self.cine_strip_count,
                    self.cine_selected_strip,
                    ACCENT,
                ) {
                    CinePreviewEvent::OffsetChanged => self.invalidate_cine_output(),
                    CinePreviewEvent::StripClicked(i) => {
                        self.cine_selected_strip = Some(i);
                        self.pick_cine_image_for_slot(&ctx, i);
                    }
                    CinePreviewEvent::None => {}
                }

                if let Some(out_tex) = &self.cine_output_texture {
                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Resultado generado").strong());
                    ui.add_space(8.0);
                    show_texture_preview(ui, out_tex, 400.0);
                }
            });
    }

    // ─── Status bar ─────────────────────────────────────────────────────────

    fn show_status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            match self.active_tab {
                AppTab::Splitter => {
                    if let Some(crop) = &self.crop_selection {
                        let mode = self.split_mode;
                        let (cx, cy, cw, ch) = crop.rect(mode);
                        ui.label(
                            egui::RichText::new(format!(
                                "Recorte: {:.0}×{:.0} px ({})  ·  {} fotos {}",
                                cw,
                                ch,
                                mode.crop_aspect_label(),
                                mode.slice_count(),
                                mode.slice_label()
                            ))
                            .small(),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!(
                                "Posición: ({:.0}, {:.0})",
                                cx,
                                cy
                            ))
                            .small()
                            .weak(),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("Abre una imagen horizontal para empezar")
                                .small()
                                .weak(),
                        );
                    }
                }
                AppTab::Polaroid => {
                    if let Some(placement) = &self.polaroid_placement {
                        let (w, h) = self.polaroid_format.output_size();
                        ui.label(
                            egui::RichText::new(format!(
                                "{}×{}  ·  Zoom: {:.0}%  ·  Rotación: {:.0}°",
                                w,
                                h,
                                placement.zoom * 100.0,
                                placement.rotation_deg
                            ))
                            .small(),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("Abre una imagen para empezar")
                                .small()
                                .weak(),
                        );
                    }
                }
                AppTab::Cine => {
                    let (w, h) = self.cine_canvas.output_size();
                    let n = self.cine_strip_count.as_usize();
                    let filled = self.cine_filled_count();
                    let (sw, sh) = self.cine_canvas.strip_output_size(self.cine_strip_count);
                    ui.label(
                        egui::RichText::new(format!(
                            "{w}×{h}  ·  {filled}/{n} imágenes  ·  {sw}×{sh} px por tira"
                        ))
                        .small(),
                    );
                }
            }

            if !self.status.is_empty() {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(&self.status).small().weak());
                });
            }
        });
    }
}

// ─── eframe App ─────────────────────────────────────────────────────────────

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar")
            .min_height(36.0)
            .max_height(36.0)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_gray(20))
                    .inner_margin(egui::Margin::symmetric(16, 0)),
            )
            .show(ctx, |ui| {
                self.show_status_bar(ui);
            });

        // Left sidebar
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(SIDEBAR_W)
            .frame(egui::Frame::new().fill(PANEL_BG).inner_margin(16.0))
            .show(ctx, |ui| {
                self.show_sidebar(ui, ctx);
            });

        // Central preview
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(PREVIEW_BG).inner_margin(12.0))
            .show(ctx, |ui| {
                self.show_preview(ui);
            });
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn sidebar_label(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).size(11.0).weak());
}

fn show_empty_state(ui: &mut egui::Ui, msg: &str) {
    let available = ui.available_size();
    ui.add_space((available.y / 2.0 - 36.0).max(40.0));
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("Sin imagen").size(22.0).weak());
        ui.add_space(6.0);
        ui.label(egui::RichText::new(msg).weak());
    });
}

fn show_mode_toggle(ui: &mut egui::Ui, active_tab: &mut AppTab) {
    let w = SIDEBAR_W - 32.0;
    let h = 34.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(w, h), egui::Sense::click());
    let painter = ui.painter();

    painter.rect_filled(rect, h / 2.0, Color32::from_gray(38));

    let third = w / 3.0;
    let sel_x = match active_tab {
        AppTab::Splitter => rect.min.x,
        AppTab::Polaroid => rect.min.x + third,
        AppTab::Cine => rect.min.x + 2.0 * third,
    };
    let sel_rect =
        Rect::from_min_size(Pos2::new(sel_x, rect.min.y), Vec2::new(third, h));
    painter.rect_filled(sel_rect, h / 2.0, ACCENT);

    for (i, label) in ["RECORTE", "POLAROID", "CINE"].iter().enumerate() {
        let is_active = match i {
            0 => *active_tab == AppTab::Splitter,
            1 => *active_tab == AppTab::Polaroid,
            _ => *active_tab == AppTab::Cine,
        };
        painter.text(
            Pos2::new(rect.min.x + third * i as f32 + third / 2.0, rect.center().y),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.0),
            if is_active {
                Color32::WHITE
            } else {
                Color32::from_gray(140)
            },
        );
    }

    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let rel = pos.x - rect.min.x;
            *active_tab = if rel < third {
                AppTab::Splitter
            } else if rel < 2.0 * third {
                AppTab::Polaroid
            } else {
                AppTab::Cine
            };
        }
    }
}

fn draw_split_mode_card(
    ui: &mut egui::Ui,
    mode: SplitMode,
    selected: bool,
    size: Vec2,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let hovered = response.hovered();

        let bg = if selected {
            TEAL
        } else if hovered {
            Color32::from_gray(52)
        } else {
            CARD_BG
        };
        let stroke_color = if selected {
            Color32::from_rgb(80, 200, 150)
        } else if hovered {
            Color32::from_gray(100)
        } else {
            Color32::from_gray(58)
        };

        painter.rect_filled(rect, 8.0, bg);
        painter.rect_stroke(
            rect,
            8.0,
            egui::Stroke::new(if selected { 1.5 } else { 1.0 }, stroke_color),
            egui::StrokeKind::Inside,
        );

        // Format icon (top 55%)
        let margin = 10.0;
        let icon_area = Rect::from_min_size(
            rect.min + Vec2::new(margin, margin),
            Vec2::new(size.x - 2.0 * margin, size.y * 0.52 - margin),
        );
        let icon_fill = if selected {
            Color32::WHITE
        } else {
            Color32::from_gray(145)
        };
        draw_split_icon_in_area(&painter, icon_area, mode, icon_fill);

        // Labels (bottom 45%)
        let text_y = rect.min.y + size.y * 0.60;
        let text_color = if selected {
            Color32::WHITE
        } else {
            Color32::from_gray(210)
        };
        let sub_color = if selected {
            Color32::from_rgb(190, 245, 220)
        } else {
            Color32::from_gray(105)
        };

        let ratio = match mode {
            SplitMode::TwoSquare => "1:1",
            SplitMode::TwoPortrait => "4:5",
            SplitMode::ThreePortrait => "3×4:5",
        };
        painter.text(
            Pos2::new(rect.center().x, text_y),
            egui::Align2::CENTER_TOP,
            ratio,
            egui::FontId::proportional(13.0),
            text_color,
        );
        painter.text(
            Pos2::new(rect.center().x, rect.max.y - 10.0),
            egui::Align2::CENTER_BOTTOM,
            match mode {
                SplitMode::TwoSquare | SplitMode::TwoPortrait => "2 fotos",
                SplitMode::ThreePortrait => "3 fotos",
            },
            egui::FontId::proportional(10.0),
            sub_color,
        );
    }

    response.clicked()
}

fn draw_split_icon_in_area(
    painter: &egui::Painter,
    area: Rect,
    mode: SplitMode,
    fill: Color32,
) {
    let n = mode.slice_count();
    let crop_aspect = mode.crop_aspect();
    let area_aspect = area.width() / area.height();

    let (frame_w, frame_h) = if crop_aspect > area_aspect {
        (area.width(), area.width() / crop_aspect)
    } else {
        (area.height() * crop_aspect, area.height())
    };

    let frame_rect =
        Rect::from_center_size(area.center(), Vec2::new(frame_w, frame_h));
    let gap = 2.0;
    let slice_w = (frame_w - gap * (n as f32 - 1.0)) / n as f32;

    for i in 0..n {
        let x = frame_rect.min.x + i as f32 * (slice_w + gap);
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(x, frame_rect.min.y),
                Vec2::new(slice_w, frame_h),
            ),
            2.0,
            fill,
        );
    }
}

fn draw_polaroid_format_card(
    ui: &mut egui::Ui,
    format: PolaroidFormat,
    selected: bool,
    size: Vec2,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let hovered = response.hovered();

        let bg = if selected {
            ACCENT
        } else if hovered {
            Color32::from_gray(52)
        } else {
            CARD_BG
        };
        let stroke_color = if selected {
            Color32::from_rgb(100, 160, 255)
        } else if hovered {
            Color32::from_gray(100)
        } else {
            Color32::from_gray(58)
        };

        painter.rect_filled(rect, 8.0, bg);
        painter.rect_stroke(
            rect,
            8.0,
            egui::Stroke::new(if selected { 1.5 } else { 1.0 }, stroke_color),
            egui::StrokeKind::Inside,
        );

        // Format icon
        let margin = 10.0;
        let icon_area = Rect::from_min_size(
            rect.min + Vec2::new(margin, margin),
            Vec2::new(size.x - 2.0 * margin, size.y * 0.50 - margin),
        );

        let (out_w, out_h) = format.output_size();
        let frame_aspect = out_w as f32 / out_h as f32;
        let area_aspect = icon_area.width() / icon_area.height();
        let (fw, fh) = if frame_aspect > area_aspect {
            (icon_area.width(), icon_area.width() / frame_aspect)
        } else {
            (icon_area.height() * frame_aspect, icon_area.height())
        };
        let icon_rect =
            Rect::from_center_size(icon_area.center(), Vec2::new(fw, fh));
        let icon_fill = if selected {
            Color32::WHITE
        } else {
            Color32::from_gray(145)
        };
        painter.rect_filled(icon_rect, 2.0, icon_fill);

        let text_y = rect.min.y + size.y * 0.58;
        let text_color = if selected {
            Color32::WHITE
        } else {
            Color32::from_gray(210)
        };
        let sub_color = if selected {
            Color32::from_rgb(190, 215, 255)
        } else {
            Color32::from_gray(105)
        };

        painter.text(
            Pos2::new(rect.center().x, text_y),
            egui::Align2::CENTER_TOP,
            format.label(),
            egui::FontId::proportional(13.0),
            text_color,
        );
        painter.text(
            Pos2::new(rect.center().x, rect.max.y - 10.0),
            egui::Align2::CENTER_BOTTOM,
            format.size_label(),
            egui::FontId::proportional(9.0),
            sub_color,
        );
    }

    response.clicked()
}

fn draw_cine_canvas_card(
    ui: &mut egui::Ui,
    canvas: CineCanvas,
    selected: bool,
    size: Vec2,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let hovered = response.hovered();

        let bg = if selected {
            TEAL
        } else if hovered {
            Color32::from_gray(52)
        } else {
            CARD_BG
        };
        let stroke_color = if selected {
            Color32::from_rgb(80, 200, 150)
        } else if hovered {
            Color32::from_gray(100)
        } else {
            Color32::from_gray(58)
        };

        painter.rect_filled(rect, 8.0, bg);
        painter.rect_stroke(
            rect,
            8.0,
            egui::Stroke::new(if selected { 1.5 } else { 1.0 }, stroke_color),
            egui::StrokeKind::Inside,
        );

        let margin = 10.0;
        let icon_area = Rect::from_min_size(
            rect.min + Vec2::new(margin, margin),
            Vec2::new(size.x - 2.0 * margin, size.y * 0.50 - margin),
        );

        let (out_w, out_h) = canvas.output_size();
        let frame_aspect = out_w as f32 / out_h as f32;
        let area_aspect = icon_area.width() / icon_area.height();
        let (fw, fh) = if frame_aspect > area_aspect {
            (icon_area.width(), icon_area.width() / frame_aspect)
        } else {
            (icon_area.height() * frame_aspect, icon_area.height())
        };
        let icon_rect =
            Rect::from_center_size(icon_area.center(), Vec2::new(fw, fh));
        let icon_fill = if selected {
            Color32::WHITE
        } else {
            Color32::from_gray(145)
        };

        let n = 3usize;
        let gap = 1.5;
        let strip_h = (fh - gap * (n as f32 - 1.0)) / n as f32;
        for i in 0..n {
            let y = icon_rect.min.y + i as f32 * (strip_h + gap);
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(icon_rect.min.x, y),
                    Vec2::new(fw, strip_h),
                ),
                1.5,
                icon_fill,
            );
        }

        let text_y = rect.min.y + size.y * 0.58;
        let text_color = if selected {
            Color32::WHITE
        } else {
            Color32::from_gray(210)
        };
        let sub_color = if selected {
            Color32::from_rgb(190, 245, 220)
        } else {
            Color32::from_gray(105)
        };

        painter.text(
            Pos2::new(rect.center().x, text_y),
            egui::Align2::CENTER_TOP,
            canvas.label(),
            egui::FontId::proportional(13.0),
            text_color,
        );
        painter.text(
            Pos2::new(rect.center().x, rect.max.y - 10.0),
            egui::Align2::CENTER_BOTTOM,
            canvas.size_label(),
            egui::FontId::proportional(9.0),
            sub_color,
        );
    }

    response.clicked()
}

fn dynamic_image_to_texture(
    ctx: &egui::Context,
    name: &str,
    img: &DynamicImage,
) -> TextureHandle {
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
    ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR)
}
