use eframe::egui;
use egui::Separator;
use std::collections::HashMap;
use std::vec;

use crate::{fileexplorer, git};
use crate::lock::tag::Tag;
use crate::lock::{lockstore, tag, LfsLock};
use crate::lock::lockstore::LockStore;

type LockSortFunc = dyn FnMut(&LfsLock, &LfsLock) -> std::cmp::Ordering;

pub struct WranglerGui {
    locks: Vec<LfsLock>,
    lock_selection: HashMap<u32, bool>,
    explorer: fileexplorer::FileExplorer,
    lock_store: Box<dyn LockStore>,
    lock_sort_fn: Box<LockSortFunc>,
    // Backing search texts
    file_search: String,
}

impl Default for WranglerGui {
    fn default() -> Self {
        WranglerGui {
            locks: vec![],
            lock_selection: HashMap::<u32, bool>::new(),
            explorer: fileexplorer::FileExplorer::new(".".into()),
            //lock_store: lockstore::monothread_lockstore::MonothreadLockStore::new(),
            lock_store: lockstore::multithreaded_lockstore::MultithreadedLockStore::new(),
            lock_sort_fn: Box::new(file_sort),
            file_search: "".into(),
        }
    }
}

fn file_sort<'a, 'b>(l1: &'a LfsLock, l2: &'b LfsLock) -> std::cmp::Ordering {
    l1.file.cmp(&l2.file)
}
fn owner_sort(l1: &LfsLock, l2: &LfsLock) -> std::cmp::Ordering {
    l1.owner.cmp(&l2.owner)
}
fn id_sort(l1: &LfsLock, l2: &LfsLock) -> std::cmp::Ordering {
    l1.id.cmp(&l2.id)
}
fn branch_sort(l1: &LfsLock, l2: &LfsLock) -> std::cmp::Ordering {
    l1.branch.cmp(&l2.branch)
}
fn dir_sort(l1: &LfsLock, l2: &LfsLock) -> std::cmp::Ordering {
    l1.dir.cmp(&l2.dir)
}

impl WranglerGui {
    pub fn new(_: &eframe::CreationContext) -> Self {
        let mut gui = Self::default();
        gui.refresh_locks();
        gui
    }

    fn render_lock_headers(&mut self, ui: &mut egui::Ui) {

        ui.label("");
        ui.add(egui::TextEdit::singleline(&mut self.file_search));
        ui.add(egui::Separator::default().vertical());
        ui.end_row();

        ui.label("");
        if ui.label("Filepath").clicked() {
            self.lock_sort_fn = Box::new(file_sort);
        }
        ui.add(Separator::default().vertical());
        if ui.label("Owner").clicked() {
            self.lock_sort_fn = Box::new(owner_sort);
        }
        ui.add(Separator::default().vertical());
        if ui.label("Lock ID").clicked() {
            self.lock_sort_fn = Box::new(id_sort);
        }
        ui.add(Separator::default().vertical());
        if ui.label("Associated branch").clicked() {
            self.lock_sort_fn = Box::new(branch_sort);
        }
        ui.add(Separator::default().vertical());
        if ui.label("Associated dir").clicked() {
            self.lock_sort_fn = Box::new(dir_sort);
        }
        ui.add(Separator::default().vertical());
        ui.label("Queue");
        ui.end_row();
    }

    fn render_lock(check: &mut bool, lock: &LfsLock, ui: &mut egui::Ui) {
        ui.checkbox(check, "");
        ui.monospace(&lock.file);
        ui.add(Separator::default().vertical());
        ui.monospace(&lock.owner);
        ui.add(Separator::default().vertical());
        ui.monospace(&lock.id.to_string());
        ui.add(Separator::default().vertical());
        match &lock.branch {
            None => ui.label("No associate branch"),
            Some(name) => ui.monospace(name),
        };
        ui.add(Separator::default().vertical());
        match &lock.dir {
            None => ui.label("No associated directory"),
            Some(dir) => ui.monospace(dir),
        };
        ui.add(Separator::default().vertical());
        if lock.queue.len() == 0 {
            ui.label("No queue detected");
        } else {
            ui.monospace(format!("{:?}", lock.queue));
        }
        ui.end_row();
    }

    fn render_locks(&mut self, ui: &mut egui::Ui) {
        self.locks.sort_by( |l1, l2| (self.lock_sort_fn)(&l1, &l2));
        let file_re = match regex::Regex::new(&self.file_search) {
            Err(_) => regex::Regex::new("").expect("Failed to compile empty regex somehow"),
            Ok(r) => r,
        };
        for lock in &self.locks {
            match self.lock_selection.get_mut(&lock.id) {
                None => (),
                Some(b) => {
                    if file_re.is_match(&lock.file) {
                        Self::render_lock(b, &lock, ui)
                    }
                },
            }
        }
    }

    pub fn release_locks(&self) {
        for (id, selected) in &self.lock_selection {
            if *selected {
                self.lock_store.unlock_id_fast(*id);
            }
        }
    }

    fn refresh_locks<'b>(&'b mut self) {
        self.lock_selection = HashMap::<u32, bool>::new();
        self.lock_store.update();
        self.locks = self.lock_store.get_locks().into_iter().filter(|lock| !git::is_lock_test(lock)).collect();
        for lock in &self.locks {
            self.lock_selection.insert(lock.id, false);
        }
        self.explorer.refresh_locks();
    }
}

impl eframe::App for WranglerGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("file explorer").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.explorer.render(ui) {
                    self.refresh_locks();
                }
            });
        });
        egui::TopBottomPanel::bottom("Control Buttons").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Release locks")).clicked() {
                    self.release_locks();
                    self.refresh_locks();
                }
                if ui.button("Enqueue for locks").clicked() {
                    for (id, sel) in &self.lock_selection {
                        if *sel {
                            match self.locks.iter_mut().find(|lock| lock.id == *id) {
                                Some(mut lock) => {
                                    let queue_tag = tag::queuetag::for_lock(&lock);
                                    queue_tag.tag(&mut lock, &*self.lock_store);
                                },
                                None => (),
                            }
                        }
                    }
                }
                if ui.button("Sync locks").clicked() {
                    self.refresh_locks();
                }
                if ui.button("Dequeue for locks").clicked() {
                    for (id, sel) in &self.lock_selection {
                        if *sel {
                            match self.locks.iter_mut().find(|lock| lock.id == *id) {
                                Some(lock) => {
                                    let queue_tag = tag::queuetag::for_lock(&lock);
                                    queue_tag.delete(&*self.lock_store);
                                    lock.queue.retain(|v| *v != git::get_lfs_user());
                                },
                                None => (),
                            }
                        }
                    }
                }
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_height_range(100.0..=500.0);
                egui::Grid::new("lfs lock view").show(ui, |ui| {
                    self.render_lock_headers(ui);
                    self.render_locks(ui);
                });
            });
        });
    }
}