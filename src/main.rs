#![feature(test)]
#![feature(iterator_step_by)]
#![feature(duration_extras)]
#![windows_subsystem = "windows"]

extern crate ggez;
extern crate strsim;
extern crate test;
extern crate winapi;

pub mod app;
pub mod cars;
pub mod definitions;
pub mod graphs;
pub mod util;

use app::*;
use definitions::*;
use ggez::*;
use std::env;
use std::mem;
use std::path;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::*;
use winapi::um::memoryapi::*;
use winapi::um::winnt::*;

// pub const MAP_OBJECT_NAME: &str = "$pcars2$";
pub const MAP_OBJECT_NAME: [u16; 9] = [36, 112, 99, 97, 114, 115, 50, 36, 0];

fn main() {
    let file_handle =
        unsafe { OpenFileMappingW(PAGE_READONLY, 0, (&MAP_OBJECT_NAME) as *const u16) };

    if file_handle.is_null() {
        println!("Game is not open!");
        return;
    }

    let size_of = mem::size_of::<SharedMemory>() as usize;

    let shared_data: *const SharedMemory =
        unsafe { MapViewOfFile(file_handle, PAGE_READONLY, 0, 0, size_of) as *const SharedMemory };

    if shared_data.is_null() {
        unsafe {
            println!(
                "Shared data is invalid, check versions.\nError code: [{:?}]",
                GetLastError()
            );
            CloseHandle(file_handle);
        };
        return;
    }

    // println!("Shared memory version: {:?}", unsafe {
    //     (*shared_data).mVersion
    // });

    if unsafe { (*shared_data).mVersion } != SHARED_MEMORY_VERSION {
        println!("Data version mismatch!");
        return;
    }

    let mut cb = ContextBuilder::new("power-graph", "ggez")
        .window_setup(
            conf::WindowSetup::default()
                .title("Don\'t take names seriously")
                .samples(4)
                .unwrap(),
        )
        .window_mode(conf::WindowMode::default().dimensions(1200, 600));

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        // println!("Adding path {:?}", path);
        cb = cb.add_resource_path(path);
    } else {
        // println!("Not building from cargo?  Ok.");
    }

    let ctx = &mut cb.build().unwrap();

    let state = &mut PC2App::new(ctx, shared_data, 1200f32, 600f32, 20);
    event::run(ctx, state).unwrap();
}
