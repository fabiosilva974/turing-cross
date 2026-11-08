@echo off
title Turing Smart Screen 3.5 Controller
cd /d "%~dp0"
if exist "target\release\turing-cross.exe" (
    start "" "target\release\turing-cross.exe"
) else (
    echo Executavel nao encontrado. Compile com: cargo build --release
    pause
)

