//! Renderizador de Framebuffer 480x320 e conversor RGB565 para o tema Turing

use crate::hardware::TuringMetrics;
use crate::protocol::{FRAMEBUFFER_SIZE, HEIGHT, WIDTH};

#[inline(always)]
pub fn rgb_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    let r5 = ((r as u16) >> 3) & 0x1F;
    let g6 = ((g as u16) >> 2) & 0x3F;
    let b5 = ((b as u16) >> 3) & 0x1F;
    (r5 << 11) | (g6 << 5) | b5
}

pub struct ScreenRenderer {
    pub framebuffer: Vec<u8>,
    pub pixels_rgb: Vec<u8>, // Para o preview na interface egui (RGBA ou RGB888)
}

impl ScreenRenderer {
    pub fn new() -> Self {
        Self {
            framebuffer: vec![0u8; FRAMEBUFFER_SIZE],
            pixels_rgb: vec![0u8; (WIDTH * HEIGHT * 3) as usize],
        }
    }

    /// Renderiza o tema "Turing" completo idêntico à captura de tela
    pub fn render_turing_theme(&mut self, metrics: &TuringMetrics) {
        // Limpa a tela com preto
        for i in 0..((WIDTH * HEIGHT) as usize) {
            self.pixels_rgb[i * 3] = 10;
            self.pixels_rgb[i * 3 + 1] = 10;
            self.pixels_rgb[i * 3 + 2] = 10;
        }

        // 1. Desenha barras decorativas verde-limão e cabeçalho sci-fi
        // Linhas superiores
        self.draw_rect(0, 0, WIDTH, 4, 180, 255, 0);
        self.draw_rect(10, 15, 120, 28, 25, 30, 25);
        self.draw_rect(10, 15, 4, 28, 180, 255, 0);

        // 2. Divisores centrais e áreas dos mostradores
        // Posições dos 4 círculos
        let c1_x = 310;
        let c1_y = 90; // CPU
        let c2_x = 415;
        let c2_y = 90; // GPU
        let c3_x = 310;
        let c3_y = 210; // RAM
        let c4_x = 415;
        let c4_y = 210; // SSD

        // Desenha os 4 anéis circulares
        self.draw_ring(c1_x, c1_y, 42, 6, metrics.cpu_usage / 100.0, 180, 255, 0);
        self.draw_ring(c2_x, c2_y, 42, 6, metrics.gpu_usage / 100.0, 180, 255, 0);
        self.draw_ring(c3_x, c3_y, 42, 6, metrics.ram_usage / 100.0, 180, 255, 0);
        self.draw_ring(c4_x, c4_y, 42, 6, metrics.ssd_usage / 100.0, 180, 255, 0);

        // 3. Barra inferior de status
        self.draw_rect(0, 275, WIDTH, 2, 80, 90, 80);

        // Converte o buffer RGB interno para o buffer final de envio USB em RGB565 (Little-Endian)
        for i in 0..((WIDTH * HEIGHT) as usize) {
            let r = self.pixels_rgb[i * 3];
            let g = self.pixels_rgb[i * 3 + 1];
            let b = self.pixels_rgb[i * 3 + 2];
            let c565 = rgb_to_rgb565(r, g, b);

            self.framebuffer[i * 2] = (c565 & 0xFF) as u8;
            self.framebuffer[i * 2 + 1] = ((c565 >> 8) & 0xFF) as u8;
        }
    }

    fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, r: u8, g: u8, b: u8) {
        for dy in 0..h {
            let py = y + dy;
            if py >= HEIGHT { break; }
            for dx in 0..w {
                let px = x + dx;
                if px >= WIDTH { break; }
                let idx = ((py * WIDTH + px) * 3) as usize;
                self.pixels_rgb[idx] = r;
                self.pixels_rgb[idx + 1] = g;
                self.pixels_rgb[idx + 2] = b;
            }
        }
    }

    fn draw_ring(&mut self, cx: i32, cy: i32, radius: i32, thickness: i32, pct: f32, r: u8, g: u8, b: u8) {
        let r_inner = radius - thickness;
        let r_outer = radius;
        let r_inner_sq = r_inner * r_inner;
        let r_outer_sq = r_outer * r_outer;

        let sweep_angle = pct.clamp(0.0, 1.0) * std::f32::consts::TAU;

        for y in (cy - radius)..=(cy + radius) {
            if y < 0 || y >= HEIGHT as i32 { continue; }
            for x in (cx - radius)..=(cx + radius) {
                if x < 0 || x >= WIDTH as i32 { continue; }
                let dx = x - cx;
                let dy = y - cy;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq >= r_inner_sq && dist_sq <= r_outer_sq {
                    let mut angle = (dy as f32).atan2(dx as f32) + std::f32::consts::FRAC_PI_2;
                    if angle < 0.0 { angle += std::f32::consts::TAU; }

                    let (pr, pg, pb) = if angle <= sweep_angle {
                        (r, g, b)
                    } else {
                        (40, 45, 40) // Anel inativo escuro
                    };

                    let idx = ((y as u32 * WIDTH + x as u32) * 3) as usize;
                    self.pixels_rgb[idx] = pr;
                    self.pixels_rgb[idx + 1] = pg;
                    self.pixels_rgb[idx + 2] = pb;
                }
            }
        }
    }
}
