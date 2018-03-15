extern crate memmap;
extern crate winapi;

pub mod definitions;
use definitions::*;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::memoryapi::*;
use winapi::um::handleapi::*;
use winapi::um::winnt::*;
use std::mem;

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

    loop {
        let local_copy = unsafe { std::ptr::read_volatile(shared_data) };
        let update_index = local_copy.mSequenceNumber;

        if update_index % 2 == 0 {
            continue;
        }

        println!("Current index: {}", update_index);

        let is_valid_participant_index = local_copy.mViewedParticipantIndex != -1
            && local_copy.mViewedParticipantIndex < local_copy.mNumParticipants
            && local_copy.mViewedParticipantIndex < STORED_PARTICIPANTS_MAX as i32;

        if is_valid_participant_index {
            let info =
                &local_copy.mParticipantInfo.data[local_copy.mViewedParticipantIndex as usize];

            println!("Participant name: {}", info.mName.to_string());
            println!("Lap distance: {}", info.mCurrentLapDistance);
        }

        println!("Game state: {:?}", local_copy.mGameState);
        println!("Session state: {:?}", local_copy.mSessionState);
        println!("Odometer KM: {:?}", local_copy.mOdometerKM);
        println!("Car name: {}", local_copy.mCarName.to_string());

        print!("{}[2J", 27 as char);
    }
}
