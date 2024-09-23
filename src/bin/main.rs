#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::AppCreator;
use git_lfs_wrangler::{gui, lock::LfsLock};
 
fn get_gui(locks: Vec<LfsLock>) -> AppCreator {
    Box::new(|cc| Ok(Box::new({
        let mut g = gui::WranglerGui::new(cc);
        g.add_locks(locks);
        g
    }
    )))
}

fn main() {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let locks = git_lfs_wrangler::lock::get_locks();
    let mut opts = eframe::NativeOptions::default();
    opts.follow_system_theme = false;
    let _ = eframe::run_native("Git Lfs Wrangler", opts, get_gui(locks));
}