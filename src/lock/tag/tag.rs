use crate::lock::*;
use crate::lock::tag::*;

pub trait Tag {
    // Update a lock's info
    fn apply(&self, lock: &mut LfsLock);
    // Get the string associated with the backing lock
    fn get_lock_string(&self) -> String;
    // Save the relevant info to the lfs
    fn save(&self, store: &LockStore)
    {
        store.lock_file(&self.get_lock_string());
    }
    // Delete the tag's backing lock
    fn delete(&self, _store: &LockStore)
    {
        lock::LfsLock::unlock_file(&self.get_lock_string());
    }
    // Get the id of the lock this tag is associated with
    fn get_target_id(&self) -> u32;
    // Apply and save
    fn tag(&self, lock: &mut LfsLock, store: &mut LockStore) {
        self.apply(lock);
        self.save(store);
    }
    // Clean up a tag that no longer points to a given lock
    fn cleanup(&self, store: &LockStore) {
        self.delete(store);
    }
}

type TagCtor = fn(&LfsLock) -> Option<Box<dyn Tag>>;

struct TagFactory {
    ctors: &'static [TagCtor],
}

static FACTORY: &'static TagFactory = &TagFactory {
    ctors: &[dirtag::DirTag::from_lock,
    branchtag::BranchTag::from_lock,
    queuetag::QueueTag::from_lock],
};

//impl TagFactory {
    // If a lock is a tag, then we hand back a tag. If it doesn't, None
    pub fn get_tag(lock: &LfsLock) -> Option<Box<dyn Tag>> {
        for f in FACTORY.ctors {
            match f(lock) {
                None => (),
                Some(tag) => {
                    return Some(tag);
                }
            }
        }
        None
    }
//}