#![feature(rust_2018_preview, duration_extras, iterator_step_by)]
#![windows_subsystem = "windows"]

extern crate ggez;
extern crate smallvec;
extern crate strsim;
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
use std::ffi::OsStr;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::path;
use std::ptr::null_mut;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::*;
use winapi::um::memoryapi::*;
use winapi::um::winnt::*;
use winapi::um::winuser::{MessageBoxW, MB_OK};

// pub const MAP_OBJECT_NAME: &str = "$pcars2$";
// pub const MAP_OBJECT_NAME: [u16; 9] = [36, 112, 99, 97, 114, 115, 50, 36, 0];

fn main() {
    let file_name: Vec<u16> = OsStr::new("$pcars2$")
        .encode_wide()
        .chain(once(0))
        .collect();

    let file_handle = unsafe { OpenFileMappingW(PAGE_READONLY, 0, file_name.as_ptr()) };

    if file_handle.is_null() {
        print_message("Game is not open!").unwrap();
        return;
    }

    let size_of = mem::size_of::<SharedMemory>() as usize;

    let shared_data: *const SharedMemory =
        unsafe { MapViewOfFile(file_handle, PAGE_READONLY, 0, 0, size_of) as *const SharedMemory };

    if shared_data.is_null() {
        unsafe {
            let msg = format!(
                "Shared data is invalid, check versions.\nError code: [{:?}]",
                GetLastError()
            );
            CloseHandle(file_handle);
            print_message(&msg).unwrap();
        };
        return;
    }

    // println!("Shared memory version: {:?}", unsafe {
    //     (*shared_data).mVersion
    // });

    if unsafe { (*shared_data).mVersion } != SHARED_MEMORY_VERSION {
        let msg = format!(
            "Data version mismatch, found: [{}], required: [{}]",
            unsafe { (*shared_data).mVersion },
            SHARED_MEMORY_VERSION
        );
        print_message(&msg).unwrap();
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

use std::io::Error;

fn print_message(msg: &str) -> Result<i32, Error> {
    let error: Vec<u16> = OsStr::new("Error!").encode_wide().chain(once(0)).collect();
    let message: Vec<u16> = OsStr::new(msg).encode_wide().chain(once(0)).collect();

    let ret = unsafe { MessageBoxW(null_mut(), message.as_ptr(), error.as_ptr(), MB_OK) };

    if ret == 0 {
        Err(Error::last_os_error())
    } else {
        Ok(ret)
    }
}
