//! `angband/src/borg/borg.h`
use crate::headers::stdint_intn_h::int32_t;
pub use crate::borg::Borg as borg_struct;
pub use crate::borg::BorgBest as borg_best;
pub use crate::borg::BorgTemp as C2RustUnnamedH_17;
pub use crate::borg::BorgStatus as C2RustUnnamedH_16;
pub use crate::borg::BorgTimes as C2RustUnnamedH_15;
pub use crate::borg::BorgGoal as C2RustUnnamedH_14;
pub use crate::borg::BorgInitSave as C2RustUnnamedH_13;
pub use crate::borg::BorgMonInfo as C2RustUnnamedH_12;
pub use crate::borg::BorgIdInfo as C2RustUnnamedH_11;
pub use crate::borg::BorgNear as C2RustUnnamedH_10;
pub type borg_time = int32_t;
pub type C2RustUnnamedH_9 = ::core::ffi::c_uint;
// The settings enum. Since borg-rs#60 it is no longer C's: the ten
// munchkin/lunal settings the C oracle keeps are retired here
// (`borg_init::RETIRED_SETTINGS`) and the five `borg_trip_*` settings are
// appended. Settings are matched by name, never by index, across the two
// backends (`the_c_table_matches_the_rust_table`).
pub const BORG_TOWN_TARGET_DEPTH: C2RustUnnamedH_9 = 47;
pub const BORG_TOWN_TRIP_TURNS: C2RustUnnamedH_9 = 48;
pub const BORG_TOWN_W_GOLD: C2RustUnnamedH_9 = 49;
pub const BORG_TOWN_DEPTH_GAIN: C2RustUnnamedH_9 = 50;
pub const BORG_TOWN_EXPLORE_TURNS: C2RustUnnamedH_9 = 51;
pub const BORG_TOWN_STAIR_TURNS: C2RustUnnamedH_9 = 52;
pub const BORG_TOWN_FOOD_TURNS: C2RustUnnamedH_9 = 53;
pub const BORG_TOWN_LIGHT_TURNS: C2RustUnnamedH_9 = 54;
pub const BORG_TOWN_RISK: C2RustUnnamedH_9 = 55;
pub const BORG_TOWN_HOME_DISCOUNT: C2RustUnnamedH_9 = 56;
pub const BORG_TOWN_W_SPACE: C2RustUnnamedH_9 = 57;
pub const BORG_TOWN_POWER_SCALE: C2RustUnnamedH_9 = 58;
pub const BORG_MAX_SETTINGS: C2RustUnnamedH_9 = 72;
pub const BORG_TRIP_RETURN_GOLD: C2RustUnnamedH_9 = 63;
pub const BORG_TRIP_UNKNOWN_STACKS: C2RustUnnamedH_9 = 62;
pub const BORG_TRIP_PACK_SLACK: C2RustUnnamedH_9 = 61;
pub const BORG_TRIP_DEPTH_MARGIN: C2RustUnnamedH_9 = 60;
pub const BORG_TRIP_POLICY: C2RustUnnamedH_9 = 59;
pub const BORG_TOWN_FOOD_TRIP: C2RustUnnamedH_9 = 46;
pub const BORG_TOWN_LIGHT_DEPTH: C2RustUnnamedH_9 = 45;
pub const BORG_TOWN_SHARE: C2RustUnnamedH_9 = 44;
pub const BORG_TOWN_W_TRIP: C2RustUnnamedH_9 = 43;
pub const BORG_TOWN_W_POWER: C2RustUnnamedH_9 = 42;
pub const BORG_TOWN_W_KNOWLEDGE: C2RustUnnamedH_9 = 41;
pub const BORG_TOWN_W_ESCAPE: C2RustUnnamedH_9 = 40;
pub const BORG_TOWN_W_POISON: C2RustUnnamedH_9 = 39;
pub const BORG_TOWN_W_BLEEDING: C2RustUnnamedH_9 = 38;
pub const BORG_TOWN_W_FOOD: C2RustUnnamedH_9 = 37;
pub const BORG_TOWN_W_LIGHT: C2RustUnnamedH_9 = 36;
pub const BORG_TOWN_PLAN: C2RustUnnamedH_9 = 35;
pub const BORG_TOWN_NO_STORES: C2RustUnnamedH_9 = 34;
pub const BORG_BUY_IDENTIFY: C2RustUnnamedH_9 = 33;
pub const BORG_BUY_STRATEGY_RESERVE410_CLASSES: C2RustUnnamedH_9 = 32;
pub const BORG_BUY_STRATEGY_CURRENT_CLASSES: C2RustUnnamedH_9 = 31;
pub const BORG_BUY_STRATEGY_DEFAULT: C2RustUnnamedH_9 = 30;
pub const BORG_RESTORE_IGNORE_SETTINGS: C2RustUnnamedH_9 = 29;
pub const BORG_AUTOSAVE: C2RustUnnamedH_9 = 28;
pub const BORG_ALLOW_STRANGE_OPTS: C2RustUnnamedH_9 = 27;
pub const BORG_STOP_ON_BELL: C2RustUnnamedH_9 = 26;
pub const BORG_SAVE_DEATH: C2RustUnnamedH_9 = 25;
pub const BORG_DUMP_LEVEL: C2RustUnnamedH_9 = 24;
pub const BORG_ENCHANT_LIMIT: C2RustUnnamedH_9 = 23;
pub const BORG_SELF_SCUM: C2RustUnnamedH_9 = 22;
pub const BORG_MONEY_SCUM_AMOUNT: C2RustUnnamedH_9 = 21;
pub const BORG_DELAY_FACTOR: C2RustUnnamedH_9 = 20;
pub const BORG_CHEST_FAIL_TOLERANCE: C2RustUnnamedH_9 = 19;
pub const BORG_RESPAWN_RACE: C2RustUnnamedH_9 = 18;
pub const BORG_RESPAWN_CLASS: C2RustUnnamedH_9 = 17;
pub const BORG_RESPAWN_WINNERS: C2RustUnnamedH_9 = 16;
pub const BORG_CHEAT_DEATH: C2RustUnnamedH_9 = 15;
pub const BORG_STOP_KING: C2RustUnnamedH_9 = 14;
pub const BORG_NO_DEEPER: C2RustUnnamedH_9 = 13;
pub const BORG_STOP_CLEVEL: C2RustUnnamedH_9 = 12;
pub const BORG_STOP_DLEVEL: C2RustUnnamedH_9 = 11;
pub const BORG_USES_DYNAMIC_CALCS: C2RustUnnamedH_9 = 10;
pub const BORG_USES_SWAPS: C2RustUnnamedH_9 = 9;
pub const BORG_KILLS_UNIQUES: C2RustUnnamedH_9 = 8;
pub const BORG_PLAYS_RISKY: C2RustUnnamedH_9 = 7;
pub const BORG_WORSHIPS_GOLD: C2RustUnnamedH_9 = 6;
pub const BORG_WORSHIPS_AC: C2RustUnnamedH_9 = 5;
pub const BORG_WORSHIPS_MANA: C2RustUnnamedH_9 = 4;
pub const BORG_WORSHIPS_HP: C2RustUnnamedH_9 = 3;
pub const BORG_WORSHIPS_SPEED: C2RustUnnamedH_9 = 2;
pub const BORG_WORSHIPS_DAMAGE: C2RustUnnamedH_9 = 1;
pub const BORG_VERBOSE: C2RustUnnamedH_9 = 0;


pub const BORG_TOWN_W_ATTACK: C2RustUnnamedH_9 = 64;
pub const BORG_TOWN_W_MAGIC: C2RustUnnamedH_9 = 65;
pub const BORG_TOWN_W_SUPPORT: C2RustUnnamedH_9 = 66;
pub const BORG_TOWN_W_RESTORE: C2RustUnnamedH_9 = 67;
pub const BORG_TOWN_ATTACK_TURNS: C2RustUnnamedH_9 = 68;
pub const BORG_TOWN_SUPPORT_TURNS: C2RustUnnamedH_9 = 69;

pub const BORG_TOWN_W_RANGER_MISSILES: C2RustUnnamedH_9 = 70;
pub const BORG_TOWN_RANGER_MISSILE_TURNS: C2RustUnnamedH_9 = 71;
