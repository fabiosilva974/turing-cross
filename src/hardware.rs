//! Leitura de métricas do sistema para exibição na telinha Turing 3.5"

use sysinfo::{Components, Disks, Networks, System};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TuringMetrics {
    // Temperaturas (°C)
    pub cpu_temp: f32,
    pub gpu_temp: f32,
    pub ssd_temp: f32,

    // Utilização (%)
    pub cpu_usage: f32,
    pub gpu_usage: f32,
    pub ram_usage: f32,
    pub ssd_usage: f32,

    // Rede (MB/s)
    pub net_upload_mb: f32,
    pub net_download_mb: f32,

    // Data e Hora formatada
    pub datetime_str: String,
}

impl Default for TuringMetrics {
    fn default() -> Self {
        Self {
            cpu_temp: 36.0,
            gpu_temp: 39.0,
            ssd_temp: 41.0,
            cpu_usage: 82.0,
            gpu_usage: 83.0,
            ram_usage: 28.0,
            ssd_usage: 15.0,
            net_upload_mb: 80.1,
            net_download_mb: 80.1,
            datetime_str: chrono::Local::now().format("%Y/%m/%d   %H:%M").to_string(),
        }
    }
}

pub struct HardwareReader {
    sys: System,
    components: Components,
    disks: Disks,
    networks: Networks,
    last_net_check: Instant,
    last_rx_bytes: u64,
    last_tx_bytes: u64,
    pub metrics: TuringMetrics,
}

impl HardwareReader {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let components = Components::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();

        let (rx, tx) = Self::total_network_bytes(&networks);

        Self {
            sys,
            components,
            disks,
            networks,
            last_net_check: Instant::now(),
            last_rx_bytes: rx,
            last_tx_bytes: tx,
            metrics: TuringMetrics::default(),
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.components.refresh();
        self.networks.refresh();

        // CPU & RAM Usage
        self.metrics.cpu_usage = self.sys.global_cpu_usage();
        let total_mem = self.sys.total_memory() as f32;
        let used_mem = self.sys.used_memory() as f32;
        if total_mem > 0.0 {
            self.metrics.ram_usage = (used_mem / total_mem) * 100.0;
        }

        // Temperaturas via Components
        self.metrics.cpu_temp = self.read_cpu_temp();
        self.metrics.gpu_temp = self.read_gpu_temp();
        self.metrics.ssd_temp = self.read_ssd_temp();

        // GPU Usage (estimativa proporcional se sem driver proprietário)
        if self.metrics.gpu_usage == 0.0 || self.metrics.gpu_usage == 83.0 {
            self.metrics.gpu_usage = (self.metrics.gpu_temp - 30.0).max(5.0) * 1.5;
            self.metrics.gpu_usage = self.metrics.gpu_usage.clamp(0.0, 100.0);
        }

        // Disco
        self.metrics.ssd_usage = 15.0; // Valor nominal ou primeira partição
        if let Some(disk) = self.disks.iter().next() {
            let total = disk.total_space() as f32;
            let available = disk.available_space() as f32;
            if total > 0.0 {
                self.metrics.ssd_usage = ((total - available) / total) * 100.0;
            }
        }

        // Velocidade de Rede
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_net_check).as_secs_f32();
        if elapsed >= 0.5 {
            let (rx, tx) = Self::total_network_bytes(&self.networks);
            let rx_diff = rx.saturating_sub(self.last_rx_bytes);
            let tx_diff = tx.saturating_sub(self.last_tx_bytes);

            self.metrics.net_download_mb = (rx_diff as f32 / (1024.0 * 1024.0)) / elapsed;
            self.metrics.net_upload_mb = (tx_diff as f32 / (1024.0 * 1024.0)) / elapsed;

            self.last_rx_bytes = rx;
            self.last_tx_bytes = tx;
            self.last_net_check = now;
        }

        // Data / Hora
        self.metrics.datetime_str = chrono::Local::now().format("%Y/%m/%d   %H:%M").to_string();
    }

    fn total_network_bytes(networks: &Networks) -> (u64, u64) {
        let mut rx = 0;
        let mut tx = 0;
        for (_, data) in networks.iter() {
            rx += data.total_received();
            tx += data.total_transmitted();
        }
        (rx, tx)
    }

    fn read_cpu_temp(&self) -> f32 {
        for comp in &self.components {
            let label = comp.label().to_lowercase();
            if label.contains("cpu") || label.contains("core") || label.contains("tctl") || label.contains("tdie") {
                let t = comp.temperature();
                if t > 0.0 {
                    return t;
                }
            }
        }
        36.0
    }

    fn read_gpu_temp(&self) -> f32 {
        for comp in &self.components {
            let label = comp.label().to_lowercase();
            if label.contains("gpu") || label.contains("edge") || label.contains("radeon") || label.contains("nvidia") {
                let t = comp.temperature();
                if t > 0.0 {
                    return t;
                }
            }
        }
        39.0
    }

    fn read_ssd_temp(&self) -> f32 {
        for comp in &self.components {
            let label = comp.label().to_lowercase();
            if label.contains("ssd") || label.contains("nvme") || label.contains("drive") {
                let t = comp.temperature();
                if t > 0.0 {
                    return t;
                }
            }
        }
        41.0
    }
}
