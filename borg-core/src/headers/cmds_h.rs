//! `/home/user/borg-rs/angband/src/cmds.h`
pub use super::cmd_core_h::command;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn do_cmd_wiz_cure_all(cmd: *mut command);
}
