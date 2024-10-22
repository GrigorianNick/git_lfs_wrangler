use crate::lock::lockstore::LockStore;
use crate::lock::{self, LfsLock};
use crate::lock::tag::Tag;

use core::time;
use std::sync::mpsc::{self, *};

pub enum Command {
    LockReal(String),
    UnlockID(u32),
    Update,
    FetchLocks,
    Enqueue(u32),
    Dequeue(u32),
    UpdateCTX(egui::Context),
}

pub struct Daemon {
    lock_chan: Receiver<Vec<LfsLock>>,
    cmd_chan: Sender<Command>,
}

fn update_store(tx: Sender<Command>) {
    loop {
        match tx.send(Command::Update) {
            Err(_) => return,
            Ok(_) => std::thread::sleep(time::Duration::from_secs(300)),
        }
        tx.send(Command::FetchLocks).expect("tx should be valid");
    }
}

fn run_store(cmd_rx: Receiver<Command>, lock_tx: Sender<Vec<LfsLock>>) {
    let store = lock::lockstore::multithreaded_lockstore::MultithreadedLockStore::new();
    let mut ctx = None;
    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            Command::Update => store.update(),
            Command::UnlockID(id) => {
                store.unlock_id(id);
            },
            Command::LockReal(file) => { store.lock_real_file(&file);},
            Command::FetchLocks => lock_tx.send(store.get_locks()).unwrap(),
            Command::Enqueue(id) => {
                match store.get_lock_id(id) {
                    Some(lock) => {
                        let tag = lock::tag::queuetag::for_lock(&lock);
                        tag.save(&*store);
                    },
                    _ => (),
                }
            }
            Command::Dequeue(id) => {
                match store.get_lock_id(id) {
                    Some(lock) => {
                        let tag = lock::tag::queuetag::for_lock(&lock);
                        tag.delete(&*store);
                    },
                    _ => (),
                }
            }
            Command::UpdateCTX(new_ctx) => ctx = Some(new_ctx),
        }
        match ctx {
            Some(ref c) => c.request_repaint(),
            _ => (),
        }
    }
}

pub fn spawn(spawn_update_thread: bool) -> Daemon {
    let (c_tx, c_rx) = mpsc::channel();
    let (l_tx, l_rx) = mpsc::channel();
    let update_tx = c_tx.clone();
    std::thread::spawn(move || {
        run_store(c_rx, l_tx);
    });
    if spawn_update_thread {
        std::thread::spawn(move || {
            update_store(update_tx);
        });
    }
    Daemon{
        lock_chan: l_rx, 
        cmd_chan: c_tx,
    }
}

impl Daemon {
    // Blocks until new locks are handed back
    pub fn fetch_locks(&self) -> Vec<LfsLock> {
        self.cmd_chan.send(Command::FetchLocks).expect("Failed to send message!");
        self.lock_chan.recv().expect("Failed to read message!")
    }

    pub fn refresh_locks(&self) {
        self.cmd_chan.send(Command::FetchLocks).expect("Failed to send message!");
    }

    pub fn update_locks(&self) {
        self.cmd_chan.send(Command::Update).expect("Failed to send message!");
    }

    pub fn check_locks(&self) -> Option<Vec<LfsLock>> {
        match self.lock_chan.try_recv() {
            Ok(locks) => Some(locks),
            Err(_) => None,
        }
    }

    pub fn unlock_id(&self, id: u32) {
        self.cmd_chan.send(Command::UnlockID(id)).expect("Failed to send message!");
    }

    pub fn set_ctx(&self, ctx: egui::Context) {
        self.cmd_chan.send(Command::UpdateCTX(ctx)).expect("Failed to send message!");
    }

    pub fn lock_real_file(&self, p: &String) {
        self.cmd_chan.send(Command::LockReal(p.clone())).expect("Failed to send message!");
    }

    pub fn enqueue(&self, target_id: u32) {
        self.cmd_chan.send(Command::Enqueue(target_id)).expect("Failed to send message!");
    }

    pub fn dequeue(&self, target_id: u32) {
        self.cmd_chan.send(Command::Dequeue(target_id)).expect("Failed to send message!");
    }
}