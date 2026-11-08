#!/usr/bin/env bash
# Script de Instalação do Turing Smart Screen 3.5" no Linux
set -e

echo "=== Instalador Turing Smart Screen 3.5\" (Linux) ==="

# 1. Detectar Gerenciador de Pacotes e Instalar Dependências
if command -v apt &> /dev/null; then
    echo "[+] Detectado Ubuntu/Debian. Instalando dependências..."
    sudo apt update
    sudo apt install -y build-essential pkg-config libudev-dev lm-sensors
elif command -v dnf &> /dev/null; then
    echo "[+] Detectado Fedora/RHEL. Instalando dependências..."
    sudo dnf install -y gcc gcc-c++ make pkgconf-pkg-config systemd-devel lm_sensors
elif command -v pacman &> /dev/null; then
    echo "[+] Detectado Arch Linux. Instalando dependências..."
    sudo pacman -S --needed --noconfirm base-devel pkgconf systemd lm_sensors
else
    echo "[!] Gerenciador de pacotes não reconhecido. Certifique-se de ter Rust, pkg-config e libudev instalados."
fi

# 2. Configurar Regras UDEV para Acesso à Porta Serial USB sem Root
echo "[+] Configurando regras UDEV para o dispositivo USB CDC ACM (VID: 0x1A86, PID: 0x5722)..."
sudo cp "$(dirname "$0")/99-turing.rules" /etc/udev/rules.d/99-turing.rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# Adicionar usuário ao grupo dialout
USER_NAME="${SUDO_USER:-$USER}"
sudo usermod -aG dialout "$USER_NAME" 2>/dev/null || true

# 3. Compilar em Release
echo "[+] Compilando binário em modo Release..."
cd "$(dirname "$0")/.."
cargo build --release

# 4. Instalar Executável no Sistema
echo "[+] Instalando binário em /usr/local/bin/turing-cross..."
sudo cp target/release/turing-cross /usr/local/bin/
sudo chmod +x /usr/local/bin/turing-cross

# 5. Instalar Serviço do Usuário Systemd (opcional)
mkdir -p "$HOME/.config/systemd/user/"
cp "$(dirname "$0")/turing-cross.service" "$HOME/.config/systemd/user/"
systemctl --user daemon-reload

echo ""
echo "=== Instalação Concluída com Sucesso! ==="
echo "Para executar agora:"
echo "  turing-cross"
echo ""
echo "Para habilitar inicialização automática no login:"
echo "  systemctl --user enable --now turing-cross.service"
echo ""
echo "Nota: Se o seu usuário acabou de ser adicionado ao grupo dialout, faça logout e login novamente para aplicar a permissão."

