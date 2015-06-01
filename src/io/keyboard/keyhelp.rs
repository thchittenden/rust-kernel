// Lots of constants we don't want to delete in here.
#![allow(dead_code)]

use core::prelude::*;

const KH_STATE_SHIFT: u32 = 16;
const KH_STATE_SMASK: u32 = 0xff00;
const KH_RMODS_SHIFT: u32 = 16;
const KH_RMODS_SMASK: u32 = 0x000f;
const KH_RAWCHAR_SHIFT: u32 = 8;
const KH_RAWCHAR_SMASK: u32 = 0xff;
const KH_CHAR_SHIFT: u32 = 0;
const KH_CHAR_SMASK: u32 = 0xff;

const KH_LSHIFT_KEY: u32 = 0x8000;
const KH_RSHIFT_KEY: u32 = 0x4000;
const KH_LCONTROL_KEY: u32 = 0x2000;
const KH_RCONTROL_KEY: u32 = 0x1000;
const KH_LALT_KEY: u32 = 0x0800;
const KH_RALT_KEY: u32 = 0x0400;
const KH_CAPS_LOCK: u32 = 0x0200;
const KH_NUM_LOCK: u32 = 0x0100;
const KH_SHIFT_KEY: u32 = KH_LSHIFT_KEY | KH_RSHIFT_KEY;
const KH_CONTROL_KEY: u32 = KH_LCONTROL_KEY | KH_RCONTROL_KEY;
const KH_ALT_KEY: u32 = KH_LALT_KEY | KH_RALT_KEY;

const KH_RESULT_HASRAW: u32 = 0x08;
const KH_RESULT_HASDATA: u32 = 0x04;
const KH_RESULT_NUMPAD: u32 = 0x02;
const KH_RESULT_MAKE: u32 = 0x01;

macro_rules! kh_gen {
    ($id:ident, $var:ident, $res:ty, $exp:expr) => (
        #[inline]
        fn $id ($var: u32) -> $res {
            $exp
        }
    );
}

kh_gen!(kh_state, k, u32, (k >> KH_STATE_SHIFT) & KH_STATE_SMASK);
kh_gen!(kh_rmods, k, u32, (k >> KH_RMODS_SHIFT) & KH_RMODS_SMASK);
kh_gen!(kh_getraw, k, u32, (k >> KH_RAWCHAR_SHIFT) & KH_RAWCHAR_SMASK);
kh_gen!(kh_getchar, k, char, ((k >> KH_CHAR_SHIFT) & KH_CHAR_SMASK) as u8 as char);

kh_gen!(kh_capslock, k, bool, (k >> KH_STATE_SHIFT) & KH_CAPS_LOCK != 0);
kh_gen!(kh_shift, k, bool, (k >> KH_STATE_SHIFT) & KH_SHIFT_KEY != 0);
kh_gen!(kh_ctl, k, bool, (k >> KH_STATE_SHIFT) & KH_CONTROL_KEY != 0);
kh_gen!(kh_alt, k, bool, (k >> KH_STATE_SHIFT) & KH_ALT_KEY != 0);
kh_gen!(kh_hasraw, k, bool, (k >> KH_RMODS_SHIFT) & KH_RESULT_HASRAW != 0);
kh_gen!(kh_haschar, k, bool, (k >> KH_RMODS_SHIFT) & KH_RESULT_HASDATA != 0);
kh_gen!(kh_numpad, k, bool, (k >> KH_RMODS_SHIFT) & KH_RESULT_NUMPAD != 0);
kh_gen!(kh_ismake, k, bool, (k >> KH_RMODS_SHIFT) & KH_RESULT_MAKE != 0);

extern {
    fn process_scancode(scancode: u32) -> u32;
}

pub fn process_key(key: u8) -> Option<char> {
    let res = unsafe { process_scancode(key as u32) };
    if kh_ismake(res) && kh_haschar(res) {
        Some(kh_getchar(res))
    } else {
        None
    }
}
