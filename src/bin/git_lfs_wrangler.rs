#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use git_lfs_wrangler::gui;

fn main() {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let mut opts = eframe::NativeOptions::default();
    opts.follow_system_theme = false;
    let _ = eframe::run_native("Git Lfs Wrangler", opts, Box::new(|cc| Ok(Box::new(gui::WranglerGui::new(cc)))));
}