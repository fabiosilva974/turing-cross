//! Interface de Controle do Turing Smart Screen (Baseada na interface oficial)

use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use crate::hardware::TuringMetrics;

#[derive(Debug, Clone)]
pub struct TuringUiConfig {
    pub screen_flip: u8, // 0 = 0°, 1 = 90°, 2 = 180°, 3 = 270°
    pub enable_text_bg: bool,
    pub brightness: u8, // 0 a 100
    pub is_running: bool,
    pub theme_name: String,
}

impl Default for TuringUiConfig {
    fn default() -> Self {
        Self {
            screen_flip: 2, // 180° por padrão como no seu screenshot
            enable_text_bg: false,
            brightness: 70,
            is_running: true,
            theme_name: "Turing".to_string(),
        }
    }
}

pub fn render_ui(
    ui: &mut Ui,
    config: &mut TuringUiConfig,
    metrics: &TuringMetrics,
    connected: bool,
    port_name: &str,
) {
    ui.horizontal(|ui| {
        // Painel Esquerdo: Controles e Configurações
        ui.vertical(|ui| {
            ui.set_width(320.0);
            ui.spacing_mut().item_spacing.y = 14.0;

            // Status de Conexão USB
            ui.horizontal(|ui| {
                let status_dot = if connected { Color32::from_rgb(0, 220, 100) } else { Color32::RED };
                ui.painter().circle_filled(ui.cursor().min + Vec2::new(8.0, 8.0), 5.0, status_dot);
                ui.add_space(18.0);
                if connected {
                    ui.label(egui::RichText::new(format!("Conectado: {}", port_name)).color(Color32::DARK_GREEN).strong());
                } else {
                    ui.label(egui::RichText::new("Buscando tela na porta COM3...").color(Color32::DARK_RED));
                }
            });

            // Screen Flip
            ui.horizontal(|ui| {
                ui.label("Screen Flip");
                ui.add_space(20.0);
                egui::ComboBox::from_id_salt("flip_combo")
                    .selected_text(match config.screen_flip {
                        1 => "90°",
                        2 => "180°",
                        3 => "270°",
                        _ => "0°",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut config.screen_flip, 0, "0°");
                        ui.selectable_value(&mut config.screen_flip, 1, "90°");
                        ui.selectable_value(&mut config.screen_flip, 2, "180°");
                        ui.selectable_value(&mut config.screen_flip, 3, "270°");
                    });

                ui.checkbox(&mut config.enable_text_bg, "Enable Text Background");
            });

            // Theme
            ui.horizontal(|ui| {
                ui.label("Theme");
                ui.add_space(40.0);
                egui::ComboBox::from_id_salt("theme_combo")
                    .selected_text(&config.theme_name)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut config.theme_name, "Turing".to_string(), "Turing");
                        ui.selectable_value(&mut config.theme_name, "Cyberpunk2077".to_string(), "Cyberpunk2077");
                        ui.selectable_value(&mut config.theme_name, "MagicBlue".to_string(), "MagicBlue");
                        ui.selectable_value(&mut config.theme_name, "AORUS".to_string(), "AORUS");
                    });
            });

            // Brightness Slider
            ui.horizontal(|ui| {
                ui.label("Brightness");
                ui.add_space(15.0);
                ui.label("Max");
                let mut br = config.brightness as f32;
                if ui.add(egui::Slider::new(&mut br, 0.0..=100.0).show_value(false)).changed() {
                    config.brightness = br as u8;
                }
                ui.label("Min");
            });

            // Status Bar Styles
            ui.horizontal(|ui| {
                ui.label("FrontStatuBarStyle");
                ui.add_space(5.0);
                ui.label("CustomColor");
                let (rect, _) = ui.allocate_exact_size(Vec2::new(30.0, 18.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(0, 240, 255));
            });

            ui.horizontal(|ui| {
                ui.label("BackStatuBarStyle");
                ui.add_space(6.0);
                ui.label("Transparent");
            });

            ui.add_space(20.0);
            ui.horizontal(|ui| {
                let _ = ui.button("Background Setting");
                let _ = ui.button("Normal Setting");
            });
        });

        ui.add_space(20.0);

        // Painel Central: Preview da tela física 480x320
        ui.vertical(|ui| {
            render_lcd_preview(ui, metrics);
        });

        ui.add_space(20.0);

        // Painel Direito: Botões Run / Stop e Theme Editor
        ui.vertical(|ui| {
            ui.set_width(120.0);
            ui.spacing_mut().item_spacing.y = 18.0;

            let run_btn = egui::Button::new("Run").min_size(Vec2::new(110.0, 42.0));
            if ui.add(run_btn).clicked() {
                config.is_running = true;
            }

            let stop_btn = egui::Button::new("Stop").min_size(Vec2::new(110.0, 42.0));
            if ui.add(stop_btn).clicked() {
                config.is_running = false;
            }

            ui.add_space(40.0);
            let theme_btn = egui::Button::new("Theme Editor").min_size(Vec2::new(110.0, 42.0));
            let _ = ui.add(theme_btn);
        });
    });
}

fn render_lcd_preview(ui: &mut Ui, m: &TuringMetrics) {
    let preview_size = Vec2::new(420.0, 280.0);
    let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());
    let painter = ui.painter();

    // Fundo da telinha LCD (Preto)
    painter.rect_filled(rect, 4.0, Color32::from_rgb(5, 5, 5));
    painter.rect_stroke(rect, 4.0, Stroke::new(2.0_f32, Color32::from_rgb(180, 255, 0)));

    // Cabeçalho sci-fi verde limão
    let top_banner = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(130.0, 26.0));
    painter.rect_filled(top_banner, 2.0, Color32::from_rgb(25, 30, 25));
    painter.text(top_banner.center(), egui::Align2::CENTER_CENTER, "图灵智显", egui::FontId::proportional(15.0), Color32::from_rgb(180, 255, 0));

    painter.text(rect.min + Vec2::new(preview_size.x - 12.0, 16.0), egui::Align2::RIGHT_CENTER, "TUNING SMART SCREEN", egui::FontId::proportional(11.0), Color32::from_rgb(180, 255, 0));

    // Bloco esquerdo: Temperaturas
    let left_x = rect.min.x + 20.0;
    let base_y = rect.min.y + 70.0;

    painter.text(Pos2::new(left_x, base_y), egui::Align2::LEFT_CENTER, "温度", egui::FontId::proportional(13.0), Color32::from_rgb(200, 200, 200));
    painter.text(Pos2::new(left_x + 40.0, base_y), egui::Align2::LEFT_CENTER, format!("CPU  {:.0} ℃", m.cpu_temp), egui::FontId::proportional(15.0), Color32::from_rgb(180, 255, 0));
    painter.text(Pos2::new(left_x + 40.0, base_y + 35.0), egui::Align2::LEFT_CENTER, format!("GPU  {:.0} ℃", m.gpu_temp), egui::FontId::proportional(15.0), Color32::from_rgb(180, 255, 0));
    painter.text(Pos2::new(left_x + 40.0, base_y + 70.0), egui::Align2::LEFT_CENTER, format!("SSD  {:.0} ℃", m.ssd_temp), egui::FontId::proportional(15.0), Color32::from_rgb(180, 255, 0));

    // Bloco direito: Mostradores em Anel
    let rx1 = rect.min.x + 265.0;
    let rx2 = rect.min.x + 355.0;
    let ry1 = rect.min.y + 90.0;
    let ry2 = rect.min.y + 185.0;

    render_mini_gauge(painter, Pos2::new(rx1, ry1), m.cpu_usage, "CPU");
    render_mini_gauge(painter, Pos2::new(rx2, ry1), m.gpu_usage, "GPU");
    render_mini_gauge(painter, Pos2::new(rx1, ry2), m.ram_usage, "RAM");
    render_mini_gauge(painter, Pos2::new(rx2, ry2), m.ssd_usage, "SSD");

    // Rodapé: Rede e Data/Hora
    let foot_y = rect.max.y - 28.0;
    painter.text(Pos2::new(left_x, foot_y - 14.0), egui::Align2::LEFT_CENTER, format!("上传 : {:.1}", m.net_upload_mb), egui::FontId::proportional(13.0), Color32::WHITE);
    painter.text(Pos2::new(left_x, foot_y + 8.0), egui::Align2::LEFT_CENTER, format!("下载 : {:.1}  MB/s", m.net_download_mb), egui::FontId::proportional(13.0), Color32::from_rgb(180, 255, 0));

    painter.text(Pos2::new(rect.max.x - 16.0, foot_y + 8.0), egui::Align2::RIGHT_CENTER, &m.datetime_str, egui::FontId::proportional(12.0), Color32::WHITE);
}

fn render_mini_gauge(painter: &egui::Painter, center: Pos2, val: f32, label: &str) {
    let r = 32.0;
    painter.circle_stroke(center, r, Stroke::new(5.0_f32, Color32::from_rgb(40, 45, 40)));

    let sweep = (val / 100.0).clamp(0.0, 1.0) * std::f32::consts::TAU;
    let steps = 24;
    for i in 0..steps {
        let a1 = -std::f32::consts::FRAC_PI_2 + (i as f32 / steps as f32) * sweep;
        let a2 = -std::f32::consts::FRAC_PI_2 + ((i + 1) as f32 / steps as f32) * sweep;
        let p1 = Pos2::new(center.x + r * a1.cos(), center.y + r * a1.sin());
        let p2 = Pos2::new(center.x + r * a2.cos(), center.y + r * a2.sin());
        painter.line_segment([p1, p2], Stroke::new(5.0_f32, Color32::from_rgb(180, 255, 0)));
    }

    painter.text(Pos2::new(center.x, center.y - 4.0), egui::Align2::CENTER_CENTER, format!("{:.0}", val), egui::FontId::proportional(19.0), Color32::WHITE);
    painter.text(Pos2::new(center.x, center.y + 15.0), egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(11.0), Color32::from_rgb(180, 255, 0));
}
