use std::fs;
use std::fs::DirEntry;
use lock::lockstore::LockStore;

use crate::lock::{self, lockstore, tag};

pub struct FileExplorer {
    selected_files: Vec<std::path::PathBuf>,
    cwd: std::path::PathBuf,
    locked_files: Vec<std::path::PathBuf>,
    lock_store: Box<dyn LockStore>,
}

impl Default for FileExplorer {
    fn default() -> Self {
        FileExplorer::new(".".to_string())
    }
}

impl FileExplorer {

    pub fn new(path: String) -> Self {
        let mut fs = FileExplorer {
            selected_files: vec![],
            cwd: std::path::Path::new(&path).to_path_buf(),
            locked_files: vec![],
            //lock_store: lockstore::monothread_lockstore::MonothreadLockStore::new(),
            lock_store: lockstore::multithreaded_lockstore::MultithreadedLockStore::new(),
        };
        fs.refresh_locks();
        fs
    }

    pub fn refresh_locks(&mut self) {
        let locks = self.lock_store.get_locks();
        let mut lock_paths = vec![];
        for lock in locks {
            let fixed_path = [".", &lock.file].join("/");
            lock_paths.push(std::path::Path::new(&fixed_path).to_path_buf());
        }
        self.locked_files = lock_paths;
    }

    fn render_dir_entry(&mut self, ui: &mut egui::Ui, f: &DirEntry) {
        if self.selected_files.contains(&f.path()) {
            return;
        }
        let os_name = f.file_name();
        let name = os_name.to_string_lossy();
        if f.path().is_dir() {
            ui.label("D");
            if ui.monospace(name.to_owned()).clicked() {
                self.cwd = f.path();
            }
        } else if self.locked_files.contains(&f.path()) {
            ui.label("L");
            ui.monospace(name.to_owned());
        }
        else {
            ui.label(" ");
            if ui.monospace(name).clicked() {
                self.selected_files.push(f.path());
            }
        }
        ui.end_row();
    }

    // true means we did something with locking
    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let mut should_update_locks = false;
        ui.label(&self.cwd.to_string_lossy().to_string());
        ui.separator();
        ui.horizontal(|ui| {
            egui::Grid::new("File Explorer Unselected").show(ui, |ui| {
                match self.cwd.parent() {
                    None => (),
                    Some(parent) => {
                        ui.label("D");
                        if ui.label("..").clicked() {
                            self.cwd = parent.to_path_buf();
                        }
                        ui.end_row();
                    }
                }
                for entry in fs::read_dir(&self.cwd) {
                    for e in entry {
                        match e {
                            Err(_) => println!("Not a real file"),
                            Ok(f) => {
                                self.render_dir_entry(ui, &f);
                            }
                        }
                    }
                }
            });
            ui.add(egui::Separator::default().vertical());
            egui::Grid::new("File Explorer Selected").show(ui, |ui| {
                let mut paths_to_remove: Vec<std::path::PathBuf> = vec![];
                for file in &self.selected_files {
                    let p = file.to_str().unwrap();
                    if ui.monospace(p).clicked() {
                        paths_to_remove.push(file.to_path_buf());
                    }
                    ui.end_row();
                }
                for p in &paths_to_remove {
                    self.selected_files.retain(|e| e != p);
                }
            });
        });
        ui.separator();
        if ui.button("Lock files").clicked() {
            for file in &self.selected_files {
                self.lock_store.lock_real_file(&file.to_string_lossy().to_string());
            }
            self.selected_files.clear();
            should_update_locks = true;
        }
        if should_update_locks {
            self.refresh_locks();
        }
        should_update_locks
    }
}