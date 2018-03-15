#![allow(bad_style)]

use std;
use std::fmt;

pub const SHARED_MEMORY_VERSION: u32 = 9;
pub const STRING_LENGTH_MAX: usize = 64;
pub const STORED_PARTICIPANTS_MAX: usize = 64;
pub const TYRE_COMPOUND_NAME_LENGTH_MAX: usize = 40;
pub const TYRE_MAX: usize = 4;
pub const VEC_MAX: usize = 3;

#[repr(u32)]
#[derive(Debug, Clone)]
pub enum Tyre {
  TyreFrontLeft,
  TyreFrontRight,
  TyreRearLeft,
  TyreRearRight,
  //--------------
  TyreMax,
}

#[repr(u32)]
#[derive(Debug, Clone)]
pub enum GameState {
  GAME_EXITED,
  GAME_FRONT_END,
  GAME_INGAME_PLAYING,
  GAME_INGAME_PAUSED,
  GAME_INGAME_INMENU_TIME_TICKING,
  GAME_INGAME_RESTARTING,
  GAME_INGAME_REPLAY,
  GAME_FRONT_END_REPLAY,
  //-------------
  GAME_MAX,
}

#[repr(u32)]
#[derive(Debug, Clone)]
pub enum SessionState {
  SESSION_INVALID,
  SESSION_PRACTICE,
  SESSION_TEST,
  SESSION_QUALIFY,
  SESSION_FORMATION_LAP,
  SESSION_RACE,
  SESSION_TIME_ATTACK,
  //-------------
  SESSION_MAX,
}

// (Type#3) RaceState (to be used with 'mRaceState' and 'mRaceStates')
#[repr(u32)]
#[derive(Debug, Clone)]
pub enum RaceState {
  RACESTATE_INVALID,
  RACESTATE_NOT_STARTED,
  RACESTATE_RACING,
  RACESTATE_FINISHED,
  RACESTATE_DISQUALIFIED,
  RACESTATE_RETIRED,
  RACESTATE_DNF,
  //-------------
  RACESTATE_MAX,
}

// (Type#5) Flag Colours (to be used with 'mHighestFlagColour')
#[repr(u32)]
#[derive(Debug, Clone)]
pub enum FlagColour {
  FLAG_COLOUR_NONE = 0, // Not used for actual flags, only for some query functions
  FLAG_COLOUR_GREEN,    // End of danger zone, or race started
  FLAG_COLOUR_BLUE,     // Faster car wants to overtake the participant
  FLAG_COLOUR_WHITE_SLOW_CAR, // Slow car in area
  FLAG_COLOUR_WHITE_FINAL_LAP, // Final Lap
  FLAG_COLOUR_RED,      // Huge collisions where one or more cars become wrecked and block the track
  FLAG_COLOUR_YELLOW,   // Danger on the racing surface itself
  FLAG_COLOUR_DOUBLE_YELLOW, // Danger that wholly or partly blocks the racing surface
  FLAG_COLOUR_BLACK_AND_WHITE, // Unsportsmanlike conduct
  FLAG_COLOUR_BLACK_ORANGE_CIRCLE, // Mechanical Failure
  FLAG_COLOUR_BLACK,    // Participant disqualified
  FLAG_COLOUR_CHEQUERED, // Chequered flag
  //-------------
  FLAG_COLOUR_MAX,
}

// (Type#6) Flag Reason (to be used with 'mHighestFlagReason')
#[repr(u32)]
#[derive(Debug, Clone)]
pub enum FlagReason {
  FLAG_REASON_NONE = 0,
  FLAG_REASON_SOLO_CRASH,
  FLAG_REASON_VEHICLE_CRASH,
  FLAG_REASON_VEHICLE_OBSTRUCTION,
  //-------------
  FLAG_REASON_MAX,
}

// (Type#7) Pit Mode (to be used with 'mPitMode')
#[repr(u32)]
#[derive(Debug, Clone)]
pub enum PitMode {
  PIT_MODE_NONE = 0,
  PIT_MODE_DRIVING_INTO_PITS,
  PIT_MODE_IN_PIT,
  PIT_MODE_DRIVING_OUT_OF_PITS,
  PIT_MODE_IN_GARAGE,
  PIT_MODE_DRIVING_OUT_OF_GARAGE,
  //-------------
  PIT_MODE_MAX,
}

// (Type#8) Pit Stop Schedule (to be used with 'mPitSchedule')

#[repr(u32)]
#[derive(Debug, Clone)]
pub enum PitSchedule {
  PIT_SCHEDULE_NONE = 0,           // Nothing scheduled
  PIT_SCHEDULE_PLAYER_REQUESTED,   // Used for standard pit sequence - requested by player
  PIT_SCHEDULE_ENGINEER_REQUESTED, // Used for standard pit sequence - requested by engineer
  PIT_SCHEDULE_DAMAGE_REQUESTED, // Used for standard pit sequence - requested by engineer for damage
  PIT_SCHEDULE_MANDATORY, // Used for standard pit sequence - requested by engineer from career enforced lap number
  PIT_SCHEDULE_DRIVE_THROUGH, // Used for drive-through penalty
  PIT_SCHEDULE_STOP_GO,   // Used for stop-go penalty
  PIT_SCHEDULE_PITSPOT_OCCUPIED, // Used for drive-through when pitspot is occupied
  //-------------
  PIT_SCHEDULE_MAX,
}

#[derive(Debug, Clone)]
pub struct CarFlags {
  data: u32,
}

impl CarFlags {
  pub fn headlight_on(&self) -> bool {
    self.data & (1 << 0) > 0
  }
  pub fn engine_active(&self) -> bool {
    self.data & (1 << 1) > 0
  }
  pub fn engine_warning(&self) -> bool {
    self.data & (1 << 2) > 0
  }
  pub fn speed_limiter_on(&self) -> bool {
    self.data & (1 << 3) > 0
  }
  pub fn abs_on(&self) -> bool {
    self.data & (1 << 4) > 0
  }
  pub fn handbrake_on(&self) -> bool {
    self.data & (1 << 5) > 0
  }
}

// (Type#9) Car Flags (to be used with 'mCarFlags')
// #[repr(u32)]
// #[derive(Debug, Clone)]
// pub enum CarFlags
// {
//   CAR_HEADLIGHT         = (1<<0),
//   CAR_ENGINE_ACTIVE     = (1<<1),
//   CAR_ENGINE_WARNING    = (1<<2),
//   CAR_SPEED_LIMITER     = (1<<3),
//   CAR_ABS               = (1<<4),
//   CAR_HANDBRAKE         = (1<<5),
// }

#[derive(Debug, Clone)]
pub struct TyreFlags {
  data: u32,
}

impl TyreFlags {
  pub fn is_attached(&self) -> bool {
    self.data & (1 << 0) > 0
  }

  pub fn is_inflated(&self) -> bool {
    self.data & (1 << 1) > 0
  }

  pub fn is_on_ground(&self) -> bool {
    self.data & (1 << 2) > 0
  }
}

// (Type#11) Terrain Materials (to be used with 'mTerrain')
#[repr(u32)]
#[derive(Debug, Clone)]
pub enum TerrainType {
  TERRAIN_ROAD = 0,
  TERRAIN_LOW_GRIP_ROAD,
  TERRAIN_BUMPY_ROAD1,
  TERRAIN_BUMPY_ROAD2,
  TERRAIN_BUMPY_ROAD3,
  TERRAIN_MARBLES,
  TERRAIN_GRASSY_BERMS,
  TERRAIN_GRASS,
  TERRAIN_GRAVEL,
  TERRAIN_BUMPY_GRAVEL,
  TERRAIN_RUMBLE_STRIPS,
  TERRAIN_DRAINS,
  TERRAIN_TYREWALLS,
  TERRAIN_CEMENTWALLS,
  TERRAIN_GUARDRAILS,
  TERRAIN_SAND,
  TERRAIN_BUMPY_SAND,
  TERRAIN_DIRT,
  TERRAIN_BUMPY_DIRT,
  TERRAIN_DIRT_ROAD,
  TERRAIN_BUMPY_DIRT_ROAD,
  TERRAIN_PAVEMENT,
  TERRAIN_DIRT_BANK,
  TERRAIN_WOOD,
  TERRAIN_DRY_VERGE,
  TERRAIN_EXIT_RUMBLE_STRIPS,
  TERRAIN_GRASSCRETE,
  TERRAIN_LONG_GRASS,
  TERRAIN_SLOPE_GRASS,
  TERRAIN_COBBLES,
  TERRAIN_SAND_ROAD,
  TERRAIN_BAKED_CLAY,
  TERRAIN_ASTROTURF,
  TERRAIN_SNOWHALF,
  TERRAIN_SNOWFULL,
  TERRAIN_DAMAGED_ROAD1,
  TERRAIN_TRAIN_TRACK_ROAD,
  TERRAIN_BUMPYCOBBLES,
  TERRAIN_ARIES_ONLY,
  TERRAIN_ORION_ONLY,
  TERRAIN_B1RUMBLES,
  TERRAIN_B2RUMBLES,
  TERRAIN_ROUGH_SAND_MEDIUM,
  TERRAIN_ROUGH_SAND_HEAVY,
  TERRAIN_SNOWWALLS,
  TERRAIN_ICE_ROAD,
  TERRAIN_RUNOFF_ROAD,
  TERRAIN_ILLEGAL_STRIP,
  TERRAIN_PAINT_CONCRETE,
  TERRAIN_PAINT_CONCRETE_ILLEGAL,
  TERRAIN_RALLY_TARMAC,

  //-------------
  TERRAIN_MAX,
}

// (Type#12) Crash Damage State  (to be used with 'mCrashState')
#[repr(u32)]
#[derive(Debug, Clone)]
pub enum CrashDamageState {
  CRASH_DAMAGE_NONE = 0,
  CRASH_DAMAGE_OFFTRACK,
  CRASH_DAMAGE_LARGE_PROP,
  CRASH_DAMAGE_SPINNING,
  CRASH_DAMAGE_ROLLING,
  //-------------
  CRASH_MAX,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ParticipantInfo {
  pub mIsActive: bool,
  pub mName: StringArray<u8>,    // [ string ]
  pub mWorldPosition: Vec3<f32>, // [ UNITS = World Space  X  Y  Z ]
  pub mCurrentLapDistance: f32,  // [ UNITS = Metres ]   [ RANGE = 0.0f->... ]    [ UNSET = 0.0f ]
  pub mRacePosition: u32,        // [ RANGE = 1->... ]   [ UNSET = 0 ]
  pub mLapsCompleted: u32,       // [ RANGE = 0->... ]   [ UNSET = 0 ]
  pub mCurrentLap: i32,          // [ RANGE = 0->... ]   [ UNSET = 0 ]
  pub mCurrentSector: i32,       // [ RANGE = 0->... ]   [ UNSET = -1 ]
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct StringArray<T> {
  pub data: [T; STRING_LENGTH_MAX],
}

#[derive(Clone)]
pub struct ParticipantsArray<T> {
  pub data: [T; STORED_PARTICIPANTS_MAX],
}

#[derive(Clone)]
pub struct TyresArray<T> {
  pub data: [T; TYRE_MAX],
}

#[derive(Clone, Debug)]
pub struct Vec3<T> {
  pub x: T,
  pub y: T,
  pub z: T,
}

pub type PCString = StringArray<u8>;

impl PCString {
  pub fn to_string(&self) -> String {
    let v = self
      .data
      .iter()
      .map(|b| *b)
      .take_while(|b| *b != 0u8)
      .collect::<Vec<_>>();
    let st = std::ffi::CString::new(v).unwrap_or(std::ffi::CString::new("").unwrap());
    st.into_string().unwrap()
  }

  pub fn is_empty(&self) -> bool {
    self.data[0] == 0u8
  }
}

impl fmt::Debug for PCString {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let st = self.to_string();
    st.fmt(f)
  }
}

impl<T: fmt::Debug> fmt::Debug for ParticipantsArray<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    self.data[..].fmt(f)
  }
}

impl<T: fmt::Debug> fmt::Debug for TyresArray<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    self.data[..].fmt(f)
  }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SharedMemory {
  // Version Number
  pub mVersion: u32,            // [ RANGE = 0->... ]
  pub mBuildVersionNumber: u32, // [ RANGE = 0->... ]   [ UNSET = 0 ]
  // Game States
  pub mGameState: GameState,       // [ enum (Type#1) Game state ]
  pub mSessionState: SessionState, // [ enum (Type#2) Session state ]
  pub mRaceState: RaceState,       // [ enum (Type#3) Race State ]

  // Participant Info
  pub mViewedParticipantIndex: i32, // [ RANGE = 0->STORED_PARTICIPANTS_MAX ]   [ UNSET = -1 ]
  pub mNumParticipants: i32,        // [ RANGE = 0->STORED_PARTICIPANTS_MAX ]   [ UNSET = -1 ]
  pub mParticipantInfo: ParticipantsArray<ParticipantInfo>, // [ struct (Type#13) ParticipantInfo struct ]

  // Unfiltered Input
  pub mUnfilteredThrottle: f32, // [ RANGE = 0.0f->1.0f ]
  pub mUnfilteredBrake: f32,    // [ RANGE = 0.0f->1.0f ]
  pub mUnfilteredSteering: f32, // [ RANGE = -1.0f->1.0f ]
  pub mUnfilteredClutch: f32,   // [ RANGE = 0.0f->1.0f ]

  // Vehicle information
  pub mCarName: PCString,      // [ string ]
  pub mCarClassName: PCString, // [ string ]

  // Event information
  pub mLapsInEvent: u32,         // [ RANGE = 0->... ]   [ UNSET = 0 ]
  pub mTrackLocation: PCString,  // [ string ] - untranslated shortened English name
  pub mTrackVariation: PCString, // [ string ]- untranslated shortened English variation description
  pub mTrackLength: f32,         // [ UNITS = Metres ]   [ RANGE = 0.0f->... ]    [ UNSET = 0.0f ]

  // Timings
  pub mNumSectors: i32,                 // [ RANGE = 0->... ]   [ UNSET = -1 ]
  pub mLapInvalidated: bool, // [ UNITS = boolean ]   [ RANGE = false->true ]   [ UNSET = false ]
  pub mBestLapTime: f32,     // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mLastLapTime: f32,     // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = 0.0f ]
  pub mCurrentTime: f32,     // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = 0.0f ]
  pub mSplitTimeAhead: f32,  // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mSplitTimeBehind: f32, // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mSplitTime: f32,       // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = 0.0f ]
  pub mEventTimeRemaining: f32, // [ UNITS = milli-seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mPersonalFastestLapTime: f32, // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mWorldFastestLapTime: f32, // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mCurrentSector1Time: f32,  // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mCurrentSector2Time: f32,  // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mCurrentSector3Time: f32,  // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mFastestSector1Time: f32,  // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mFastestSector2Time: f32,  // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mFastestSector3Time: f32,  // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mPersonalFastestSector1Time: f32, // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mPersonalFastestSector2Time: f32, // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mPersonalFastestSector3Time: f32, // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mWorldFastestSector1Time: f32, // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mWorldFastestSector2Time: f32, // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]
  pub mWorldFastestSector3Time: f32, // [ UNITS = seconds ]   [ RANGE = 0.0f->... ]   [ UNSET = -1.0f ]

  // Flags
  pub mHighestFlagColour: u32, // [ enum (Type#5) Flag Colour ]
  pub mHighestFlagReason: u32, // [ enum (Type#6) Flag Reason ]

  // Pit Info
  pub mPitMode: PitMode,         // [ enum (Type#7) Pit Mode ]
  pub mPitSchedule: PitSchedule, // [ enum (Type#8) Pit Stop Schedule ]

  // Car State
  pub mCarFlags: CarFlags, // [ enum (Type#9) Car Flags ]
  pub mOilTempCelsius: f32,
  pub mOilPressureKPa: f32,
  pub mWaterTempCelsius: f32,
  pub mWaterPressureKPa: f32,
  pub mFuelPressureKPa: f32,
  pub mFuelLevel: f32,
  pub mFuelCapacity: f32,
  pub mSpeed: f32,
  pub mRpm: f32,
  pub mMaxRPM: f32,
  pub mBrake: f32,
  pub mThrottle: f32,
  pub mClutch: f32,
  pub mSteering: f32,
  pub mGear: i32,
  pub mNumGears: i32,
  pub mOdometerKM: f32,
  pub mAntiLockActive: bool,
  pub mLastOpponentCollisionIndex: i32,
  pub mLastOpponentCollisionMagnitude: f32,
  pub mBoostActive: bool,
  pub mBoostAmount: f32,

  pub mOrientation: Vec3<f32>,
  pub mLocalVelocity: Vec3<f32>,
  pub mWorldVelocity: Vec3<f32>,
  pub mAngularVelocity: Vec3<f32>,
  pub mLocalAcceleration: Vec3<f32>,
  pub mWorldAcceleration: Vec3<f32>,
  pub mExtentsCentre: Vec3<f32>,

  pub mTyreFlags: TyresArray<u32>,
  pub mTerrain: TyresArray<u32>,
  pub mTyreY: TyresArray<f32>,
  pub mTyreRPS: TyresArray<f32>,
  pub mTyreSlipSpeed: TyresArray<f32>,
  pub mTyreTemp: TyresArray<f32>,
  pub mTyreGrip: TyresArray<f32>,
  pub mTyreHeightAboveGround: TyresArray<f32>,
  pub mTyreLateralStiffness: TyresArray<f32>,
  pub mTyreWear: TyresArray<f32>,
  pub mBrakeDamage: TyresArray<f32>,
  pub mSuspensionDamage: TyresArray<f32>,
  pub mBrakeTempCelsius: TyresArray<f32>,
  pub mTyreTreadTemp: TyresArray<f32>,
  pub mTyreLayerTemp: TyresArray<f32>,
  pub mTyreCarcassTemp: TyresArray<f32>,
  pub mTyreRimTemp: TyresArray<f32>,
  pub mTyreInternalAirTemp: TyresArray<f32>,

  pub mCrashState: u32,
  pub mAeroDamage: f32,
  pub mEngineDamage: f32,

  pub mAmbientTemperature: f32,
  pub mTrackTemperature: f32,
  pub mRainDensity: f32,
  pub mWindSpeed: f32,
  pub mWindDirectionX: f32,
  pub mWindDirectionY: f32,
  pub mCloudBrightness: f32,

  //volatile read here?
  pub mSequenceNumber: u32,

  //Additional car variables
  pub mWheelLocalPositionY: TyresArray<f32>,
  pub mSuspensionTravel: TyresArray<f32>,
  pub mSuspensionVelocity: TyresArray<f32>,
  pub mAirPressure: TyresArray<f32>,
  pub mEngineSpeed: f32,
  pub mEngineTorque: f32,
  pub mWings: [f32; 2],
  pub mHandBrake: f32,

  // more
  pub mCurrentSector1Times: ParticipantsArray<f32>,
  pub mCurrentSector2Times: ParticipantsArray<f32>,
  pub mCurrentSector3Times: ParticipantsArray<f32>,
  pub mFastestSector1Times: ParticipantsArray<f32>,
  pub mFastestSector2Times: ParticipantsArray<f32>,
  pub mFastestSector3Times: ParticipantsArray<f32>,
  pub mFastestLapTimes: ParticipantsArray<f32>,
  pub mLastLapTimes: ParticipantsArray<f32>,
  pub mLapsInvalidated: ParticipantsArray<bool>,
  pub mRaceStates: ParticipantsArray<u32>,
  pub mPitModes: ParticipantsArray<u32>,
  pub mOrientations: ParticipantsArray<Vec3<f32>>, // Euler Angles
  pub mSpeeds: ParticipantsArray<f32>,             // m/s
  pub mCarNames: ParticipantsArray<PCString>,
  pub mCarClassNames: ParticipantsArray<PCString>,

  // more
  pub mEnforcedPitStopLap: i32,
  pub mTranslatedTrackLocation: PCString,
  pub mTranslatedTrackVariation: PCString,
  pub mBrakeBias: f32,          // [ RANGE = 0.0f->1.0f... ]   [ UNSET = -1.0f ]
  pub mTurboBoostPressure: f32, //	 RANGE = 0.0f->1.0f... ]   [ UNSET = -1.0f ]
  pub mTyreCompound: TyresArray<PCString>, // [ strings  ]
  pub mPitSchedules: ParticipantsArray<u32>, // [ enum (Type#7)  Pit Mode ]
  pub mHighestFlagColours: ParticipantsArray<u32>, // [ enum (Type#5) Flag Colour ]
  pub mHighestFlagReasons: ParticipantsArray<u32>, // [ enum (Type#6) Flag Reason ]
  pub mNationalities: ParticipantsArray<u32>, // [ nationality table , SP AND UNSET = 0 ] See nationalities.txt file for details
  pub mSnowDensity: f32, // [ UNITS = How much snow will fall ]   [ RANGE = 0.0f->1.0f ], this will be non zero only in Snow season, in other seasons whatever is falling from the sky is reported as rain
}
