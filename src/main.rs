extern crate ggez;
extern crate memmap;
extern crate winapi;

pub mod definitions;
pub mod app;

use definitions::*;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::memoryapi::*;
use winapi::um::handleapi::*;
use winapi::um::winnt::*;
use std::mem;
use ggez::*;
use app::*;

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

    println!("Shared memory version: {:?}", unsafe {
        (*shared_data).mVersion
    });

    if unsafe { (*shared_data).mVersion } != SHARED_MEMORY_VERSION {
        println!("Data version mismatch!");
        return;
    }

    let title = String::from("Don\'t take names seriously");
    //Proceed to something useful!
    let mut c = conf::Conf::new();
    c.window_setup.title = title;
    let ctx = &mut Context::load_from_conf(title, "dp", c).unwrap();
    let state = &mut PC2App { shared_data };
    event::run(ctx, state).unwrap();

}
