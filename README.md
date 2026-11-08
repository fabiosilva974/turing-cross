# Turing Smart Screen 3.5" (IPS V2) - Cross-Platform (Rust)

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Linux & Windows](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-green.svg)]()
[![Hardware: Turing 3.5 IPS](https://img.shields.io/badge/hardware-WCH%20USB%20CDC%20ACM-blueviolet.svg)]()

Um controlador e monitor de sistema multiplataforma de altíssimo desempenho desenvolvido em **Rust** para a telinha auxiliar **Turing Smart Screen 3.5" IPS** (conhecida no mercado como *XuanFang 3.5"* com chip WCH USB-Serial CDC).

Construído através da engenharia reversa do software original `UsbMonitor.exe` (.NET v4.5), eliminando o alto consumo de CPU e memória e oferecendo suporte nativo tanto para **Linux** quanto para **Windows**.

---

## 📸 Interface e Tema

O aplicativo reproduz com fidelidade o tema visual **Turing**:
- **Temperaturas (°C)**: CPU, GPU e SSD em tempo real.
- **Mostradores em Anel (%)**: Utilização de CPU, GPU, Memória RAM e SSD.
- **Rede em Tempo Real**: Velocidade de Upload e Download em MB/s.
- **Data e Hora**: Sincronização automática com relógio do sistema.
- **Pré-visualização ao Vivo**: Simulação na janela desktop com as mesmas dimensões e gráficos da tela física.

---

## ⚡ Recursos Principais

- [x] **Comunicação USB-Serial Nativa**: Compatível com Windows (`COM3` ou qualquer porta CDC) e Linux (`/dev/ttyACM0` / `/dev/ttyUSB0`).
- [x] **Auto-Detecção de Hardware**: Identifica automaticamente a porta serial associada ao ID `USB\VID_1A86&PID_5722\USB35INCHIPSV2`.
- [x] **Motor Gráfico RGB565 Otimizado**: Renderização direta de 480x320 pixels sem depender de frameworks pesados ou .NET Framework.
- [x] **Controle de Brilho e Rotação**: Suporte a Screen Flip (0°, 90°, 180°, 270°) e ajuste fino de brilho (0% a 100%).
- [x] **Executável Único e Portátil**: Binário nativo compilado sem necessidade de runtime adicional.

---

## 🐧 Guia de Implementação no Linux

O Linux reconhece o microcontrolador WCH nativamente como um dispositivo serial USB (`cdc_acm` ou `ch341`), mapeado normalmente em `/dev/ttyACM0` ou `/dev/ttyUSB0`.

### Método 1: Instalação Automática (Recomendado)

Clone o repositório e execute o script de instalação rápida:

```bash
git clone https://github.com/fabiosilva974/turing-cross.git
cd turing-cross
chmod +x linux/install.sh
./linux/install.sh
```

O instalador cuida de:
1. Instalar as dependências do compilador e de USB/udev.
2. Copiar as regras UDEV para permitir acesso serial sem `sudo`.
3. Adicionar seu usuário ao grupo `dialout`.
4. Compilar o projeto em modo otimizado (Release).
5. Instalar o executável em `/usr/local/bin/turing-cross`.

---

### Método 2: Instalação Passo a Passo (Manual)

#### 1. Dependências do Sistema

- **Ubuntu / Debian / Pop!_OS / Linux Mint**:
  ```bash
  sudo apt update
  sudo apt install -y build-essential pkg-config libudev-dev lm-sensors
  ```

- **Fedora / RHEL**:
  ```bash
  sudo dnf install -y gcc gcc-c++ make pkgconf-pkg-config systemd-devel lm_sensors
  ```

- **Arch Linux / Manjaro**:
  ```bash
  sudo pacman -S --needed base-devel pkgconf systemd lm_sensors
  ```

#### 2. Permissão de Acesso Serial USB (UDEV)

Para comunicar com a porta serial USB sem necessidade de rodar como root (`sudo`):

1. Copie o arquivo de regra UDEV:
   ```bash
   sudo cp linux/99-turing.rules /etc/udev/rules.d/
   ```
   *(Regra: `SUBSYSTEM=="tty", ATTRS{idVendor}=="1a86", ATTRS{idProduct}=="5722", MODE="0666", GROUP="dialout"`)*

2. Recarregue as regras:
   ```bash
   sudo udevadm control --reload-rules && sudo udevadm trigger
   ```

3. Adicione seu usuário ao grupo `dialout`:
   ```bash
   sudo usermod -aG dialout $USER
   ```
   *(Efetue logout e login na sua sessão Linux para validar a alteração de grupo).*

#### 3. Compilação e Execução

Certifique-se de ter o [Rust instalado](https://rustup.rs/):
```bash
cargo build --release
./target/release/turing-cross
```

---

### ⚙️ Execução Automática no Linux (Systemd Service)

Para que a telinha inicialize automaticamente sempre que você entrar na área de trabalho:

1. Registre o serviço do usuário:
   ```bash
   mkdir -p ~/.config/systemd/user
   cp linux/turing-cross.service ~/.config/systemd/user/
   ```

2. Certifique-se de que o binário foi instalado em `/usr/local/bin`:
   ```bash
   sudo cp target/release/turing-cross /usr/local/bin/
   ```

3. Ative o serviço:
   ```bash
   systemctl --user daemon-reload
   systemctl --user enable --now turing-cross.service
   ```

---

## 🪟 Guia de Uso no Windows

1. Para executar o binário já compilado:
   - Dê um duplo clique no arquivo **`executar.bat`**.
2. Para compilar a partir do código fonte:
   ```powershell
   cargo run --release
   ```

O programa busca e conecta automaticamente na porta `COM3` (ou qualquer porta serial em que o ID `VID_1A86&PID_5722` esteja conectado).

---

## 📑 Engenharia Reversa e Protocolo

Para aprofundar-se nos detalhes de descompilação de `UsbMonitor.exe`, formato dos quadros RGB565, pacotes de handshake e limites de chunking USB, leia:
👉 [`PROTOCOLO_REVERSO.md`](../PROTOCOLO_REVERSO.md)

---

## 🛠️ Resolução de Problemas no Linux (Troubleshooting)

* **Dispositivo detectado como Modem pelo ModemManager**:
  - Em algumas distribuições, o serviço `ModemManager` pode tentar capturar portas ACM recém-conectadas por alguns segundos. A regra udev em `linux/99-turing.rules` já impede esse conflito, mas você também pode filtrar o ID caso necessário.
* **Permissão negada ao abrir `/dev/ttyACM0`**:
  - Verifique se o seu usuário pertence ao grupo `dialout` executando o comando `groups`. Se não pertencer, execute `sudo usermod -aG dialout $USER` e reinicie a sessão.
* **Sensores de Temperatura no Linux**:
  - Para leitura ideal de temperatura da CPU/GPU, certifique-se de ter o pacote `lm-sensors` configurado executando `sudo sensors-detect --auto`.

---

## 📄 Licença

Distribuído sob a licença [MIT](LICENSE).
