default: 
    @just --list

alias b := build
alias d := dev
alias da := dev-android

build: 
    cargo tauri build

dev: 
    cargo tauri dev

dev-android:
    cargo tauri android dev

build-android:
    cargo tauri android build --apk --target aarch64

clippy: 
    cd src-tauri && cargo clippy

clean: 
    cd src-tauri && cargo clean
