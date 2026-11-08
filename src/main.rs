//! Turing Smart Screen 3.5" (XuanFang) Multiplatform Controller
//! Compatível com Windows (COM3) e Linux (/dev/ttyACM0)

mod hardware;
mod protocol;
mod renderer;
mod ui;

use eframe::egui;
use hardware::HardwareReader;
use protocol::TuringDevice;
use renderer::ScreenRenderer;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use ui::{render_ui, TuringUiConfig};

struct TuringApp {
    config: TuringUiConfig,
    metrics: Arc<Mutex<hardware::TuringMetrics>>,
    connected: Arc<Mutex<bool>>,
    port_name: Arc<Mutex<String>>,
}

impl eframe::App for TuringApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(500));

        let current_metrics = {
            let m = self.metrics.lock().unwrap();
            m.clone()
        };

        let is_conn = { *self.connected.lock().unwrap() };
        let port = { self.port_name.lock().unwrap().clone() };

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(245, 245, 245)).inner_margin(24.0))
            .show(ctx, |ui| {
                render_ui(ui, &mut self.config, &current_metrics, is_conn, &port);
            });
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init();

    let mut hw = HardwareReader::new();
    hw.refresh();

    let metrics_arc = Arc::new(Mutex::new(hw.metrics.clone()));
    let metrics_thread = Arc::clone(&metrics_arc);

    let connected_arc = Arc::new(Mutex::new(false));
    let connected_thread = Arc::clone(&connected_arc);

    let port_name_arc = Arc::new(Mutex::new("Detectando...".to_string()));
    let port_name_thread = Arc::clone(&port_name_arc);

    // Thread em segundo plano: Conecta na porta USB Serial (COM3 / /dev/ttyACM0) e envia os frames
    thread::spawn(move || {
        let mut device: Option<TuringDevice> = None;
        let mut renderer = ScreenRenderer::new();

        loop {
            // Atualiza métricas
            hw.refresh();
            {
                let mut m = metrics_thread.lock().unwrap();
                *m = hw.metrics.clone();
            }

            // Conexão serial USB
            if device.is_none() {
                if let Ok(dev) = TuringDevice::auto_connect() {
                    println!("[+] Telinha conectada com sucesso na porta {}", dev.port_name);
                    {
                        *port_name_thread.lock().unwrap() = dev.port_name.clone();
                        *connected_thread.lock().unwrap() = true;
                    }
                    device = Some(dev);
                } else {
                    {
                        *connected_thread.lock().unwrap() = false;
                    }
                }
            }

            // Renderiza e transmite para o display físico
            if let Some(ref mut dev) = device {
                renderer.render_turing_theme(&hw.metrics);

                if let Err(e) = dev.send_framebuffer(&renderer.framebuffer) {
                    eprintln!("[-] Falha ao enviar para o LCD: {}. Reconectando...", e);
                    device = None;
                    {
                        *connected_thread.lock().unwrap() = false;
                    }
                }
            }

            thread::sleep(Duration::from_millis(1000));
        }
    });

    // Janela Nativa Desktop
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Turing Smart Screen")
            .with_inner_size([960.0, 420.0])
            .with_min_inner_size([900.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Turing Smart Screen",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(TuringApp {
                config: TuringUiConfig::default(),
                metrics: metrics_arc,
                connected: connected_arc,
                port_name: port_name_arc,
            }))
        }),
    )
}
