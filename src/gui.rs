use eframe::egui;
use egui::Separator;

use crate::fileexplorer;
use crate::lock::tag::Tag;
use crate::lock::{self, tag, LfsLock};

struct LfsLockModel {
    pub lock: lock::LfsLock,
    pub selected: bool,
}

#[derive(Default)]
pub struct WranglerGui {
    locks: Vec<LfsLockModel>,
    explorer: fileexplorer::FileExplorer,
}

impl WranglerGui {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        Self::default()
    }

    fn render_lock_headers(&mut self, ui: &mut egui::Ui) {
        ui.label("");
        if ui.label("Filepath").clicked() {
            self.locks.sort_by(|l1, l2| l1.lock.file.cmp(&l2.lock.file));
        }
        ui.add(Separator::default().vertical());
        if ui.label("Owner").clicked() {
            self.locks.sort_by(|l1, l2| l1.lock.owner.cmp(&l2.lock.owner));
        }
        ui.add(Separator::default().vertical());
        if ui.label("Lock ID").clicked() {
            self.locks.sort_by(|l1, l2| l1.lock.id.cmp(&l2.lock.id));
        }
        ui.add(Separator::default().vertical());
        if ui.label("Associated branch").clicked() {
            self.locks.sort_by(|l1, l2| l1.lock.branch.cmp(&l2.lock.branch));
        }
        ui.add(Separator::default().vertical());
        if ui.label("Associated dir").clicked() {
            self.locks.sort_by(|l1, l2| l1.lock.dir.cmp(&l2.lock.dir));
        }
        ui.add(Separator::default().vertical());
        ui.label("Queue");
    }

    fn render_lock(lock: &mut LfsLockModel, ui: &mut egui::Ui) {
        ui.checkbox(&mut lock.selected, "");
        ui.monospace(&lock.lock.file);
        ui.add(Separator::default().vertical());
        ui.monospace(&lock.lock.owner);
        ui.add(Separator::default().vertical());
        ui.monospace(&lock.lock.id.to_string());
        ui.add(Separator::default().vertical());
        match &lock.lock.branch {
            None => ui.label("No associate branch"),
            Some(name) => ui.monospace(name),
        };
        ui.add(Separator::default().vertical());
        match &lock.lock.dir {
            None => ui.label("No associated directory"),
            Some(dir) => ui.monospace(dir),
        };
        ui.add(Separator::default().vertical());
        if lock.lock.queue.len() == 0 {
            ui.label("No queue detected");
        } else {
            ui.monospace(format!("{:?}", lock.lock.queue));
        }
    }

    pub fn add_locks(&mut self, locks: Vec<LfsLock>) {
        for lock in locks {
            self.locks.push(LfsLockModel {
                lock: lock,
                selected: false,
            });
        }
    }

    pub fn release_locks(&self) {
        for lock in &self.locks {
            if lock.selected {
                lock.lock.unlock();
            }
        }
    }

    fn refresh_locks(&mut self) {
        self.locks.clear();
        self.add_locks(lock::get_locks());
        self.explorer.refresh_locks();
    }
}

impl eframe::App for WranglerGui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("file explorer").show(ctx, |ui| {
            if self.explorer.render(ui) {
                self.refresh_locks();
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_height_range(100.0..=500.0);
                egui::Grid::new("lfs lock view").show(ui, |ui| {
                    self.render_lock_headers(ui);
                    ui.end_row();
                    for lock in &mut self.locks {
                        WranglerGui::render_lock(lock, ui);
                        ui.end_row();
                    }
                });
            });
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Release locks")).clicked() {
                    self.release_locks();
                    self.refresh_locks();
                }
                if ui.button("Enqueue for locks").clicked() {
                    for lock in &mut self.locks {
                        if lock.selected {
                            let queue_tag = tag::queuetag::for_lock(&lock.lock);
                            queue_tag.tag(&mut lock.lock);
                        }
                    }
                    self.refresh_locks();
                }
            })
        });
    }
}