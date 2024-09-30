use crate::lock::*;

pub struct LfsLockQueue {
    file: String
}

impl LfsLockQueue {
    pub fn enqueue(lock: &LfsLock) -> LfsLockQueue {
        let queue_tag = ["Q", &lock.file].join("___");
        lock::LfsLock::lock_file(&queue_tag);
        LfsLockQueue{
            file: lock.file.to_string(),
        }
    }
}