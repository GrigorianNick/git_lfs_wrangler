#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use git_lfs_wrangler::lock::tag::Tag;
use git_lfs_wrangler::{gui, lock::lockstore};

use git_lfs_wrangler::lock::lockstore::LockStore;

use clap::Parser;
use std::process::ExitCode;

#[derive(Parser, Debug)]
/// A utility for managing git lfs lock contention
struct Cli {
    /// Locks files with helpful annotations
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    lock: Option<Vec<String>>,

    /// Unlocks files
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    unlock: Option<Vec<String>>,

    /// Enqueue for a file
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    queue: Option<Vec<String>>,

    /// Dequeue for a file
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    dequeue: Option<Vec<String>>,

    /// List the repo's locks, alongside any helpful annotations
    #[arg(long)]
    list: bool,
}

fn lock_files(locks: Vec<String>, storage: &dyn LockStore) -> bool {
    let mut success = true;
    for lock in locks {
        print!("Locking {}...", lock);
        if storage.lock_real_file(&lock).is_some() {
            println!("Success!");
        } else {
            println!("Failure!");
            success = false;
        }
    }
    storage.update();
    success
}

fn unlock_files(locks: Vec<String>, storage: &dyn LockStore) -> bool {
    let mut success = true;
    for lock in locks {
        print!("Unlocking {}...", lock);
        if storage.unlock_file(&lock) {
            println!("Success!");
        } else {
            println!("Failure!");
            success = false;
        }
    }
    storage.update();
    success
}

fn enqueue_files(target_locks: Vec<String>, storage: &dyn LockStore) -> bool {
    let mut success = true;
    let locks = storage.get_locks();
    for target_lock in target_locks {
        print!("Enqueing for {}...", target_lock);
        match locks.iter().find(|lock| lock.file == target_lock) {
            Some(lock) => {
                // We are already enqueued for it
                if lock.queue.contains(&git_lfs_wrangler::git::get_lfs_user()) {
                    println!("Already enqueued!");
                    continue;
                }
                let tag = git_lfs_wrangler::lock::tag::queuetag::for_lock(lock);
                tag.save(storage);
                println!("Success!");
            }
            None => {
                println!("Lock does not exist!");
                success = false;
            }
        }
    }
    success
}

fn dequeue_files(target_locks: Vec<String>, storage: &dyn LockStore) -> bool {
    let mut success = true;
    let locks = storage.get_locks();
    for target_lock in target_locks {
        print!("Dequeing from {}...", target_lock);
        match locks.iter().find(|lock| lock.file == target_lock) {
            Some(lock) => {
                if !lock.queue.contains(&git_lfs_wrangler::git::get_lfs_user()) {
                    success = false;
                    println!("Not enqueued for this lock!");
                    continue;
                }
                let tag = git_lfs_wrangler::lock::tag::queuetag::for_lock(lock);
                tag.delete(storage);
                println!("Success!");
            }
            None => {
                println!("Lock does not exist!");
                success = false;
            }
        }
    }
    success
}

fn main() -> ExitCode {

    let args = Cli::parse();

    let storage = *lockstore::monothread_lockstore::MonothreadLockStore::new();

    let mut cli_results = vec![];

    cli_results.push(match args.lock {
        Some(locks) => Some(lock_files(locks, &storage)),
        None => None
    });
    cli_results.push(match args.unlock {
        Some(locks) => Some(unlock_files(locks, &storage)),
        None => None,
    });
    cli_results.push(match args.queue {
        Some(locks) => Some(enqueue_files(locks, &storage)),
        None => None
    });
    cli_results.push(match args.dequeue {
        Some(locks) => Some(dequeue_files(locks, &storage)),
        None => None
    });
    cli_results.push(match args.list {
        true => {
            for lock in storage.get_locks().iter().filter(|lock| !git_lfs_wrangler::git::is_lock_test(lock)) {
                println!("{}", lock);
            }
            Some(true)
        },
        false => None,
    });

    if cli_results.contains(&Some(false)) {
        return ExitCode::FAILURE;
    } else if cli_results.contains(&Some(true)) {
        return ExitCode::SUCCESS;
    }

    let mut opts = eframe::NativeOptions::default();
    opts.follow_system_theme = false;
    let _ = eframe::run_native("Git Lfs Wrangler", opts, Box::new(|cc| Ok(Box::new(gui::WranglerGui::new(cc)))));
    ExitCode::SUCCESS
}