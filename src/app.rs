use definitions::*;
use std::collections::{BTreeMap, HashMap};
use std;
use ggez::*;
use ggez::graphics::*;

type CarName = PCString;
type TrackName = PCString;

pub struct PC2App {
    shared_data: *const SharedMemory,
    local_copy: SharedMemory,
    current_car: String,
    current_track: String,
    power_data: HashMap<TrackName, HashMap<CarName, PowerGraphData>>,
}

type ThrottleInput = f32;
type RPM = i32;
type Torque = f32;
type BoostPressure = f32;

pub struct PowerGraphData {
    data: BTreeMap<RPM, (ThrottleInput, Torque, BoostPressure)>,
}

impl PC2App {
    fn new(sm: *const SharedMemory) -> PC2App {
        PC2App {
            shared_data: sm,
            local_copy: std::ptr::read_volatile(sm),
            power_data: HashMap::new(),
            current_car: String::new(),
            current_track: String::new(),
        }
    }
}

impl event::EventHandler for PC2App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let local_copy = unsafe { std::ptr::read_volatile(self.shared_data) };
        let update_index = local_copy.mSequenceNumber;

        if update_index % 2 == 0 || self.local_copy.mSequenceNumber == local_copy.mSequenceNumber {
            return Ok(());
        }

        let is_valid_participant_index = local_copy.mViewedParticipantIndex != -1
            && local_copy.mViewedParticipantIndex < local_copy.mNumParticipants
            && local_copy.mViewedParticipantIndex < STORED_PARTICIPANTS_MAX as i32;

        if is_valid_participant_index {
            let info =
                &local_copy.mParticipantInfo.data[local_copy.mViewedParticipantIndex as usize];
        }

        let track_name = local_copy.mTrackLocation;
        let car_name = local_copy.mCarName;

        if track_name.is_empty() || car_name.is_empty() {
            return Ok(());
        }

        let current_rpm = local_copy.mRpm as i32;
        let current_torque = local_copy.mEngineTorque;
        let throttle = local_copy.mThrottle

        let current = self.power_data
            .entry(track_name)
            .or_insert(HashMap::new())
            .or_insert(HashMap::new())
            .or_insert(BTreeMap::new);

        let rpm_rounded = current_rpm - current_rpm % 10;

        println!("Game state: {:?}", local_copy.mGameState);
        println!("Session state: {:?}", local_copy.mSessionState);
        println!("Odometer KM: {:?}", local_copy.mOdometerKM);
        println!("Car name: {}", local_copy.mCarName.to_string());

        print!("{}[2J", 27 as char);

        self.local_copy = local_copy;
        Ok(())
    }
}
