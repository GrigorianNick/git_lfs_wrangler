use branchtag::BranchTag;
use dirtag::DirTag;
use queuetag::QueueTag;

use crate::lock::*;
use crate::lock::tag::*;
use crate::lock::lockstore::*;

pub trait Tag {
    // Update a lock's info
    fn apply(&self, lock: &mut LfsLock);
    // Get the string associated with the backing lock
    fn get_lock_string(&self) -> String;
    // Save the relevant info to the lfs
    fn save(&self, store: &dyn LockStore)
    {
        store.lock_file(&self.get_lock_string());
    }
    // Delete the tag's backing lock
    fn delete(&self, _store: &dyn LockStore)
    {
        lock::LfsLock::unlock_file(&self.get_lock_string());
    }
    // Get the id of the lock this tag is associated with
    fn get_target_id(&self) -> u32;
    // Apply and save
    fn tag(&self, lock: &mut LfsLock, store: &dyn LockStore) {
        self.apply(lock);
        self.save(store);
    }
    // Clean up a tag that no longer points to a given lock
    fn cleanup(&self, store: &dyn LockStore)
    {
        self.delete(store);
    }
}

pub enum Tags {
    Dir(DirTag),
    Branch(BranchTag),
    Queue(QueueTag),
}

// If a lock is a tag, then we hand back a tag. If it doesn't, None
pub fn get_tag(lock: &LfsLock) -> Option<Box<dyn Tag>> {
    match dirtag::DirTag::from_lock(lock) {
        None => (),
        Some(tag) => {
            return Some(Box::new(tag));
        },
    };
    match BranchTag::from_lock(lock) {
        None => (),
        Some(tag) => {
            return Some(Box::new(tag));
        },
    };
    match QueueTag::from_lock(lock) {
        None => None,
        Some(tag) => {
            return Some(Box::new(tag));
        }
    }
}