use std::fs;
use std::fs::DirEntry;


use super::daemon::Daemon;

pub struct FileExplorer {
    selected_files: Vec<std::path::PathBuf>,
    cwd: std::path::PathBuf,
    locked_files: Vec<std::path::PathBuf>,
    daemon: Daemon,
    //lock_store: Box<dyn LockStore>,
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
            daemon: crate::gui::daemon::spawn(false),
        };
        fs.refresh_locks();
        fs
    }

    pub fn set_ctx(&self, ctx: egui::Context) {
        self.daemon.set_ctx(ctx);
    }

    pub fn refresh_locks(&mut self) {
        self.daemon.refresh_locks();
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
        match self.daemon.check_locks() {
            Some(locks) => self.locked_files = locks.into_iter().map(|lock| {
                let fixed_path = [".", &lock.file].join("/");
                std::path::Path::new(&fixed_path).to_path_buf()
            }).collect(),
            None => (),
        }
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
                if let Ok(entry) = fs::read_dir(&self.cwd) {
                    for e in entry {
                        match e {
                            Err(_) => (),
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
                self.daemon.lock_real_file(&file.to_string_lossy().to_string());
            }
            self.refresh_locks();
            self.selected_files.clear();
            should_update_locks = true;
        }
        should_update_locks
    }
}