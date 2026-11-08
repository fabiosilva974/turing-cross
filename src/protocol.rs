//! Protocolo Serial USB do Turing Smart Screen 3.5" IPS V2
//! Chip WCH: VID 0x1A86, PID 0x5722

use serialport::{SerialPort, SerialPortType};
use std::io::{self, Write};
use std::time::Duration;

pub const WIDTH: u32 = 480;
pub const HEIGHT: u32 = 320;
pub const TOTAL_PIXELS: usize = (WIDTH * HEIGHT) as usize;
pub const FRAMEBUFFER_SIZE: usize = TOTAL_PIXELS * 2; // RGB565 = 2 bytes por pixel

pub struct TuringDevice {
    port: Box<dyn SerialPort>,
    pub port_name: String,
}

impl TuringDevice {
    /// Detecta automaticamente a telinha conectada no Windows (COM3 / USB35INCHIPSV2) ou Linux (/dev/ttyACM0)
    pub fn auto_connect() -> Result<Self, String> {
        let ports = serialport::available_ports().map_err(|e| e.to_string())?;

        for p in &ports {
            let matches = match &p.port_type {
                SerialPortType::UsbPort(info) => {
                    // VID 0x1A86, PID 0x5722 (WCH USB35INCHIPSV2)
                    info.vid == 0x1A86 && info.pid == 0x5722
                }
                _ => false,
            };

            // Fallback para nomes comuns caso o driver CDC não exponha VID/PID diretamente
            let name_lower = p.port_name.to_lowercase();
            let is_candidate = matches || name_lower.contains("com3") || name_lower.contains("ttyacm") || name_lower.contains("ttyusb");

            if is_candidate {
                if let Ok(dev) = Self::open_port(&p.port_name) {
                    println!("[+] Telinha Turing 3.5\" conectada na porta: {}", p.port_name);
                    return Ok(dev);
                }
            }
        }

        // Se a busca falhar, tenta COM3 diretamente no Windows ou /dev/ttyACM0 no Linux
        #[cfg(target_os = "windows")]
        let default_port = "COM3";
        #[cfg(not(target_os = "windows"))]
        let default_port = "/dev/ttyACM0";

        Self::open_port(default_port).map_err(|e| format!("Não foi possível conectar em {}: {}", default_port, e))
    }

    pub fn open_port(port_name: &str) -> Result<Self, io::Error> {
        let builder = serialport::new(port_name, 115_200)
            .timeout(Duration::from_millis(1000))
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One);

        let mut port = builder.open()?;
        let _ = port.write_data_terminal_ready(true);
        let _ = port.write_request_to_send(true);

        Ok(Self {
            port,
            port_name: port_name.to_string(),
        })
    }

    /// Ajusta o brilho (0 a 100)
    #[allow(dead_code)]
    pub fn set_brightness(&mut self, level_pct: u8) -> io::Result<()> {
        let level = level_pct.clamp(0, 100);
        let packet: [u8; 6] = [102, level, 0, 0, 0, 0];
        self.port.write_all(&packet)?;
        self.port.flush()?;
        Ok(())
    }

    /// Ajusta a rotação (0: Retrato 0°, 1: Paisagem 90°, 2: Retrato 180°, 3: Paisagem 270°)
    #[allow(dead_code)]
    pub fn set_rotation(&mut self, mode: u8) -> io::Result<()> {
        let packet: [u8; 6] = [108, mode % 4, 0, 0, 0, 0];
        self.port.write_all(&packet)?;
        self.port.flush()?;
        Ok(())
    }

    /// Envia o framebuffer completo de 480x320 em RGB565 (307.200 bytes)
    pub fn send_framebuffer(&mut self, rgb565_data: &[u8]) -> io::Result<()> {
        if rgb565_data.len() != FRAMEBUFFER_SIZE {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Tamanho de framebuffer inválido"));
        }

        // Cabeçalho de coordenadas para tela 480x320
        // X0=0, Y0=0, X1=479 (0x01DF), Y1=319 (0x013F)
        let header: [u8; 10] = [
            0x00, 0x00, // X0
            0x00, 0x00, // Y0
            0xDF, 0x01, // X1 (479)
            0x3F, 0x01, // Y1 (319)
            0x00, 0x00,
        ];
        self.port.write_all(&header)?;

        // Transmissão em blocos (chunks) de 64 bytes para o endpoint USB Full-Speed
        const CHUNK_SIZE: usize = 64;
        for chunk in rgb565_data.chunks(CHUNK_SIZE) {
            self.port.write_all(chunk)?;
        }
        self.port.flush()?;

        Ok(())
    }
}
