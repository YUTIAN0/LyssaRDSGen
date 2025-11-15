//! Graphical user interface with i18n support

use crate::keygen::{generate_lkp, generate_spk, validate_tskey};
use crate::types::{LicenseInfo, SPKCurve, LICENSE_TYPES};
use eframe::egui;
use num_bigint::BigUint;

#[derive(Clone, Copy, PartialEq)]
enum Language {
    English,
    Chinese,
}

struct UiText {
    title: &'static str,
    subtitle: &'static str,
    product_id: &'static str,
    product_id_hint: &'static str,
    existing_spk: &'static str,
    existing_spk_hint: &'static str,
    license_count: &'static str,
    license_type: &'static str,
    generate_spk: &'static str,
    validate_spk: &'static str,
    generate_lkp: &'static str,
    generated_keys: &'static str,
    spk_label: &'static str,
    lkp_label: &'static str,
    copy: &'static str,
    status: &'static str,
    input_params: &'static str,
    error_pid_required: &'static str,
    error_spk_required: &'static str,
    error_count_range: &'static str,
    generating_spk: &'static str,
    generating_lkp: &'static str,
    validating_spk: &'static str,
    spk_generated: &'static str,
    spk_validated: &'static str,
    spk_invalid: &'static str,
    lkp_generated: &'static str,
}

impl UiText {
    fn get(lang: Language) -> Self {
        match lang {
            Language::English => Self {
                title: "🔑 LyssaRDSGen",
                subtitle: "RDS License Key Generator",
                product_id: "Product ID",
                product_id_hint: "e.g., 00490-92005-99454-AT527",
                existing_spk: "Existing SPK (Optional)",
                existing_spk_hint: "Leave empty to generate new",
                license_count: "License Count",
                license_type: "License Type",
                generate_spk: "🔐 Generate SPK",
                validate_spk: "✓ Validate SPK",
                generate_lkp: "📦 Generate LKP",
                generated_keys: "✨ Generated Keys",
                spk_label: "License Server ID (SPK)",
                lkp_label: "License Key Pack (LKP)",
                copy: "📋 Copy",
                status: "Status",
                input_params: "📝 Input Parameters",
                error_pid_required: "Error: PID is required",
                error_spk_required: "Error: SPK is required for validation",
                error_count_range: "Error: Count must be between 1 and 9999",
                generating_spk: "Generating SPK...",
                generating_lkp: "Generating LKP...",
                validating_spk: "Validating SPK...",
                spk_generated: "SPK generated successfully!",
                spk_validated: "SPK validation successful!",
                spk_invalid: "Error: SPK does not match the PID",
                lkp_generated: "LKP generated successfully!",
            },
            Language::Chinese => Self {
                title: "🔑 LyssaRDSGen",
                subtitle: "RDS 许可证密钥生成器",
                product_id: "产品 ID",
                product_id_hint: "例如：00490-92005-99454-AT527",
                existing_spk: "现有 SPK（可选）",
                existing_spk_hint: "留空以生成新密钥",
                license_count: "许可证数量",
                license_type: "许可证类型",
                generate_spk: "🔐 生成 SPK",
                validate_spk: "✓ 验证 SPK",
                generate_lkp: "📦 生成 LKP",
                generated_keys: "✨ 生成的密钥",
                spk_label: "许可证服务器 ID (SPK)",
                lkp_label: "许可证密钥包 (LKP)",
                copy: "📋 复制",
                status: "状态",
                input_params: "📝 输入参数",
                error_pid_required: "错误：需要产品 ID",
                error_spk_required: "错误：验证需要 SPK",
                error_count_range: "错误：数量必须在 1 到 9999 之间",
                generating_spk: "正在生成 SPK...",
                generating_lkp: "正在生成 LKP...",
                validating_spk: "正在验证 SPK...",
                spk_generated: "SPK 生成成功！",
                spk_validated: "SPK 验证成功！",
                spk_invalid: "错误：SPK 与 PID 不匹配",
                lkp_generated: "LKP 生成成功！",
            },
        }
    }
}

pub struct LyssaRDSGenApp {
    pid: String,
    spk: String,
    count: u32,
    selected_license: usize,
    generated_spk: String,
    generated_lkp: String,
    status_message: String,
    is_generating: bool,
    language: Language,
}

impl Default for LyssaRDSGenApp {
    fn default() -> Self {
        Self {
            pid: String::new(),
            spk: String::new(),
            count: 1,
            selected_license: 18, // Default to Windows Server 2022 Per Device
            generated_spk: String::new(),
            generated_lkp: String::new(),
            status_message: String::new(),
            is_generating: false,
            language: Language::Chinese,
        }
    }
}

impl LyssaRDSGenApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure fonts to support Chinese characters
        let mut fonts = egui::FontDefinitions::default();
        
        // Add Noto Sans CJK font for Chinese support
        fonts.font_data.insert(
            "noto_sans_cjk".to_owned(),
            egui::FontData::from_static(include_bytes!("../fonts/NotoSansCJK-VF.ttc")),
        );
        
        // Put the Chinese font first in the list so it's used for Chinese characters
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "noto_sans_cjk".to_owned());
        
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "noto_sans_cjk".to_owned());
        
        cc.egui_ctx.set_fonts(fonts);
        
        Self::default()
    }

    fn generate_spk_clicked(&mut self, text: &UiText) {
        if self.pid.trim().is_empty() {
            self.status_message = text.error_pid_required.to_string();
            return;
        }

        self.is_generating = true;
        self.status_message = text.generating_spk.to_string();

        match generate_spk(&self.pid) {
            Ok(spk) => {
                self.generated_spk = spk;
                self.status_message = text.spk_generated.to_string();
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }

        self.is_generating = false;
    }

    fn validate_spk_clicked(&mut self, text: &UiText) {
        if self.pid.trim().is_empty() {
            self.status_message = text.error_pid_required.to_string();
            return;
        }

        if self.spk.trim().is_empty() {
            self.status_message = text.error_spk_required.to_string();
            return;
        }

        self.is_generating = true;
        self.status_message = text.validating_spk.to_string();

        match validate_tskey(
            &self.pid,
            &self.spk,
            SPKCurve::gx(),
            SPKCurve::gy(),
            SPKCurve::kx(),
            SPKCurve::ky(),
            BigUint::from(SPKCurve::A),
            SPKCurve::p(),
            true,
        ) {
            Ok(true) => {
                self.status_message = text.spk_validated.to_string();
            }
            Ok(false) => {
                self.status_message = text.spk_invalid.to_string();
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }

        self.is_generating = false;
    }

    fn generate_lkp_clicked(&mut self, text: &UiText) {
        if self.pid.trim().is_empty() {
            self.status_message = text.error_pid_required.to_string();
            return;
        }

        let count = self.count;
        if !(1..=9999).contains(&count) {
            self.status_message = text.error_count_range.to_string();
            return;
        }

        let license_type = LICENSE_TYPES[self.selected_license].0;
        let license_info = match LicenseInfo::parse(license_type) {
            Ok(info) => info,
            Err(e) => {
                self.status_message = format!("Error: {}", e);
                return;
            }
        };

        self.is_generating = true;
        self.status_message = text.generating_lkp.to_string();

        match generate_lkp(
            &self.pid,
            count,
            license_info.chid,
            license_info.major_ver,
            license_info.minor_ver,
        ) {
            Ok(lkp) => {
                self.generated_lkp = lkp;
                self.status_message = format!(
                    "{} ({})",
                    text.lkp_generated,
                    license_info.description
                );
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }

        self.is_generating = false;
    }
}

impl eframe::App for LyssaRDSGenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let text = UiText::get(self.language);

        // Apply custom styling
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(16.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(15.0);
        style.visuals.widgets.noninteractive.bg_stroke.width = 1.0;
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(245, 247, 250);
        style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(250, 251, 252);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(59, 130, 246);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(96, 165, 250);
        style.visuals.window_rounding = egui::Rounding::same(12.0);
        style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
        style.visuals.widgets.active.rounding = egui::Rounding::same(8.0);
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Header with language switcher
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(
                            egui::RichText::new(text.title)
                                .size(32.0)
                                .color(egui::Color32::from_rgb(59, 130, 246))
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new(text.subtitle)
                                .size(16.0)
                                .color(egui::Color32::from_rgb(107, 114, 128)),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Show CURRENT language (what is selected)
                        let lang_text = match self.language {
                            Language::English => "🌐 English",  // Currently English, show English
                            Language::Chinese => "🌐 中文",      // Currently Chinese, show Chinese
                        };
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new(lang_text).size(14.0))
                                    .fill(egui::Color32::from_rgb(243, 244, 246))
                                    .stroke(egui::Stroke::new(
                                        1.0,
                                        egui::Color32::from_rgb(209, 213, 219),
                                    )),
                            )
                            .clicked()
                        {
                            self.language = match self.language {
                                Language::English => Language::Chinese,
                                Language::Chinese => Language::English,
                            };
                        }
                    });
                });

                ui.add_space(20.0);

                // Input section with card style
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(255, 255, 255))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(229, 231, 235),
                    ))
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::same(20.0))
                    .shadow(egui::epaint::Shadow {
                        offset: egui::vec2(0.0, 2.0),
                        blur: 8.0,
                        spread: 0.0,
                        color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 10),
                    })
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(text.input_params)
                                .size(18.0)
                                .strong()
                                .color(egui::Color32::from_rgb(31, 41, 55)),
                        );
                        ui.add_space(15.0);

                        // Product ID
                        ui.label(
                            egui::RichText::new(text.product_id)
                                .size(14.0)
                                .color(egui::Color32::from_rgb(75, 85, 99)),
                        );
                        ui.add_space(5.0);
                        ui.add_sized(
                            [ui.available_width(), 32.0],
                            egui::TextEdit::singleline(&mut self.pid)
                                .hint_text(text.product_id_hint)
                        );

                        ui.add_space(12.0);

                        // Existing SPK
                        ui.label(
                            egui::RichText::new(text.existing_spk)
                                .size(14.0)
                                .color(egui::Color32::from_rgb(75, 85, 99)),
                        );
                        ui.add_space(5.0);
                        ui.add_sized(
                            [ui.available_width(), 32.0],
                            egui::TextEdit::singleline(&mut self.spk)
                                .hint_text(text.existing_spk_hint)
                        );

                        ui.add_space(12.0);

                        // License Count
                        ui.label(
                            egui::RichText::new(text.license_count)
                                .size(14.0)
                                .color(egui::Color32::from_rgb(75, 85, 99)),
                        );
                        ui.add_space(5.0);
                        let mut count_str = self.count.to_string();
                        ui.add_sized(
                            [ui.available_width(), 32.0],
                            egui::TextEdit::singleline(&mut count_str)
                                .hint_text("1-9999")
                        );
                        // Parse the count string back to u32
                        if let Ok(parsed) = count_str.parse::<u32>() {
                            if (1..=9999).contains(&parsed) {
                                self.count = parsed;
                            }
                        }

                        ui.add_space(12.0);

                        // License Type
                        ui.label(
                            egui::RichText::new(text.license_type)
                                .size(14.0)
                                .color(egui::Color32::from_rgb(75, 85, 99)),
                        );
                        ui.add_space(5.0);
                        egui::ComboBox::from_id_source("license_type")
                            .selected_text(LICENSE_TYPES[self.selected_license].1)
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                for (idx, (_, desc)) in LICENSE_TYPES.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut self.selected_license,
                                        idx,
                                        *desc,
                                    );
                                }
                            });
                    });

                ui.add_space(20.0);

                // Action buttons with modern styling
                ui.horizontal(|ui| {
                    let button_height = 40.0;

                    if ui
                        .add_sized(
                            [ui.available_width() / 3.0 - 10.0, button_height],
                            egui::Button::new(
                                egui::RichText::new(text.generate_spk)
                                    .size(14.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(59, 130, 246))
                            .stroke(egui::Stroke::NONE),
                        )
                        .clicked()
                        && !self.is_generating
                    {
                        self.generate_spk_clicked(&text);
                    }

                    ui.add_space(5.0);

                    if ui
                        .add_sized(
                            [ui.available_width() / 2.0 - 5.0, button_height],
                            egui::Button::new(
                                egui::RichText::new(text.validate_spk)
                                    .size(14.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(16, 185, 129))
                            .stroke(egui::Stroke::NONE),
                        )
                        .clicked()
                        && !self.is_generating
                    {
                        self.validate_spk_clicked(&text);
                    }

                    ui.add_space(5.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), button_height],
                            egui::Button::new(
                                egui::RichText::new(text.generate_lkp)
                                    .size(14.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(139, 92, 246))
                            .stroke(egui::Stroke::NONE),
                        )
                        .clicked()
                        && !self.is_generating
                    {
                        self.generate_lkp_clicked(&text);
                    }
                });

                ui.add_space(20.0);

                // Output section with card style
                if !self.generated_spk.is_empty() || !self.generated_lkp.is_empty() {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(240, 253, 244))
                        .stroke(egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgb(167, 243, 208),
                        ))
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(20.0))
                        .shadow(egui::epaint::Shadow {
                            offset: egui::vec2(0.0, 2.0),
                            blur: 8.0,
                            spread: 0.0,
                            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 10),
                        })
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(text.generated_keys)
                                    .size(18.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(6, 78, 59)),
                            );
                            ui.add_space(15.0);

                            if !self.generated_spk.is_empty() {
                                ui.label(
                                    egui::RichText::new(text.spk_label)
                                        .size(14.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(22, 101, 52)),
                                );
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    egui::Frame::none()
                                        .fill(egui::Color32::WHITE)
                                        .stroke(egui::Stroke::new(
                                            1.0,
                                            egui::Color32::from_rgb(209, 213, 219),
                                        ))
                                        .rounding(egui::Rounding::same(6.0))
                                        .inner_margin(egui::Margin::same(12.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(&self.generated_spk)
                                                    .size(13.0)
                                                    .color(egui::Color32::from_rgb(22, 101, 52))
                                                    .family(egui::FontFamily::Monospace),
                                            );
                                        });
                                    if ui
                                        .button(
                                            egui::RichText::new(text.copy)
                                                .size(13.0)
                                                .color(egui::Color32::WHITE),
                                        )
                                        .clicked()
                                    {
                                        ui.output_mut(|o| {
                                            o.copied_text = self.generated_spk.clone()
                                        });
                                    }
                                });
                                ui.add_space(12.0);
                            }

                            if !self.generated_lkp.is_empty() {
                                ui.label(
                                    egui::RichText::new(text.lkp_label)
                                        .size(14.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(22, 101, 52)),
                                );
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    egui::Frame::none()
                                        .fill(egui::Color32::WHITE)
                                        .stroke(egui::Stroke::new(
                                            1.0,
                                            egui::Color32::from_rgb(209, 213, 219),
                                        ))
                                        .rounding(egui::Rounding::same(6.0))
                                        .inner_margin(egui::Margin::same(12.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(&self.generated_lkp)
                                                    .size(13.0)
                                                    .color(egui::Color32::from_rgb(22, 101, 52))
                                                    .family(egui::FontFamily::Monospace),
                                            );
                                        });
                                    if ui
                                        .button(
                                            egui::RichText::new(text.copy)
                                                .size(13.0)
                                                .color(egui::Color32::WHITE),
                                        )
                                        .clicked()
                                    {
                                        ui.output_mut(|o| {
                                            o.copied_text = self.generated_lkp.clone()
                                        });
                                    }
                                });
                            }
                        });

                    ui.add_space(15.0);
                }

                // Status message with enhanced styling
                if !self.status_message.is_empty() {
                    let (bg_color, border_color, text_color) =
                        if self.status_message.starts_with("Error")
                            || self.status_message.contains("错误")
                        {
                            (
                                egui::Color32::from_rgb(254, 242, 242),
                                egui::Color32::from_rgb(252, 165, 165),
                                egui::Color32::from_rgb(153, 27, 27),
                            )
                        } else {
                            (
                                egui::Color32::from_rgb(240, 253, 244),
                                egui::Color32::from_rgb(167, 243, 208),
                                egui::Color32::from_rgb(22, 101, 52),
                            )
                        };

                    egui::Frame::none()
                        .fill(bg_color)
                        .stroke(egui::Stroke::new(1.0, border_color))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(&self.status_message)
                                    .size(14.0)
                                    .color(text_color),
                            );
                        });
                }

                ui.add_space(10.0);

                // Footer
                ui.separator();
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("LyssaRDSGen v1.0.0")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(156, 163, 175)),
                    );
                });
                ui.add_space(10.0);
            });
        });
    }
}

pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([750.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "LyssaRDSGen - RDS License Key Generator",
        options,
        Box::new(|cc| Box::new(LyssaRDSGenApp::new(cc))),
    )
}
