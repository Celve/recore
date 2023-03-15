use core::arch::{asm, global_asm};

use alloc::sync::Arc;
use lazy_static::lazy_static;

use crate::{
    mm::{address::VirAddr, frame::Area, memory::MappingPermission},
    sync::up::UpCell,
};

global_asm!(include_str!("trampoline.s"));

// global_asm!(
// ".altmacro",
// ".macro STORE_REG n",
// "    sd x\n, \n*8(sp)",
// ".endm",
// ".macro LOAD_REG n",
// "    ld x\n, \n*8(sp)",
// ".endm"
// );

lazy_static! {
    pub static ref TRAMPOLINE: Arc<UpCell<Area>> = {
        extern "C" {
            fn strampoline();
            fn etrampoline();
        }
        let area = Area::new_identical(
            VirAddr::from(strampoline as usize).floor_to_vir_page_num(),
            VirAddr::from(etrampoline as usize).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::X,
        );
        Arc::new(UpCell::new(area))
    };
}
