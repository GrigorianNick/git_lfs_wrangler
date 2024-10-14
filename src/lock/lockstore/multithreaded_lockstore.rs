use std::sync::mpsc;
use std::thread;

use crate::lock::{lockstore::LockStore, LfsLock};

use super::monothread_lockstore::MonothreadLockStore;

enum Request {
    GetLocks(mpsc::Sender<Vec<LfsLock>>),
    Update,
    LockFile(String, Option<mpsc::Sender<Option<LfsLock>>>),
    UnlockFile(String, Option<mpsc::Sender<bool>>),
    UnlockId(u32, Option<mpsc::Sender<bool>>),
}

fn handle_request(request: Request, store: &impl LockStore) {
    match request {
        Request::GetLocks(tx) => {
            tx.send(store.get_locks()).unwrap();
        },
        Request::LockFile(file, tx_opt) => {
            match tx_opt {
                None => {
                    store.lock_file_fast(&file);
                }
                Some(tx) => {
                    tx.send(store.lock_file_fetch(&file)).unwrap();
                }
            };
        },
        Request::Update => {
            store.update();
        },
        Request::UnlockFile(file, tx_opt) => {
            match tx_opt {
                None => store.unlock_file_fast(&file),
                Some(tx) => tx.send(store.unlock_file(&file)).unwrap(),
            }
        },
        Request::UnlockId(id, tx_opt) => {
            match tx_opt {
                None => store.unlock_id_fast(id),
                Some(tx) => tx.send(store.unlock_id(id)).unwrap()
            }
        },
        _ => (),
    }
}

fn run(chan: mpsc::Receiver<Request>) {
    let store = MonothreadLockStore::new();
    loop {
        match chan.recv() {
            Err(_) => return,
            Ok(request) => {
                handle_request(request, &*store);
            }
        }
    }
}

pub struct MultithreadedLockStore {
    chan: mpsc::Sender<Request>,
}

impl MultithreadedLockStore {
    pub fn new() -> Box<MultithreadedLockStore> {
        let (tx, rx) = mpsc::channel();
        let ls = MultithreadedLockStore{
            chan: tx,
        };
        thread::spawn(move || (run(rx)));
        Box::new(ls)
    }
}

impl LockStore for MultithreadedLockStore {
    fn get_raw_locks(&self) -> Vec<crate::lock::LfsLock> {
        let (tx, rx) = mpsc::channel();
        self.chan.send(Request::GetLocks(tx)).unwrap();
        match rx.recv() {
            Err(_) => vec![],
            Ok(locks) => locks,
        }
    }

    fn update(&self) {
        match self.chan.send(Request::Update) {
            _ => ()
        }
    }

    fn lock_file_fast(&self, p: &String) {
        self.chan.send(Request::LockFile(p.clone(), None)).unwrap();
    }

    fn unlock_file(&self, p: &String) -> bool {
        let (tx, rx) = mpsc::channel();
        self.chan.send(Request::UnlockFile(p.clone(), Some(tx))).unwrap();
        match rx.recv() {
            Err(_) => false,
            Ok(success) => success,
        }
    }

    fn unlock_file_fast(&self, p: &String) {
        self.chan.send(Request::UnlockFile(p.clone(), None)).unwrap();
    }

    fn unlock_id(&self, id: u32) -> bool {
        let (tx, rx) = mpsc::channel();
        self.chan.send(Request::UnlockId(id, Some(tx))).unwrap();
        match rx.recv() {
            Err(_) => false,
            Ok(success) => success,
        }
    }
}