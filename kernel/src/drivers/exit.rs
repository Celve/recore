use core::arch::asm;

use crate::config::VIRT_TEST;

const EXIT_SUCCESS: u32 = 0x5555; // Equals `exit(0)`. qemu successful exit
const EXIT_FAILURE_FLAG: u32 = 0x3333;
const EXIT_FAILURE: u32 = exit_code_encode(1); // Equals `exit(1)`. qemu failed exit
const EXIT_RESET: u32 = 0x7777; // qemu reset

/// Encode the exit code using EXIT_FAILURE_FLAG.
const fn exit_code_encode(code: u32) -> u32 {
    (code << 16) | EXIT_FAILURE_FLAG
}

/// A handler that handles exit in QEMU.
pub struct QemuExit {
    /// Address of the sifive_test mapped device.
    addr: u64,
}

pub const QEMU_EXIT: QemuExit = QemuExit::new(VIRT_TEST as u64);

impl QemuExit {
    pub const fn new(addr: u64) -> Self {
        QemuExit { addr }
    }

    /// Exit qemu with specified exit code.
    pub fn exit(&self, code: u32) -> ! {
        // If code is not a special value, we need to encode it with EXIT_FAILURE_FLAG.
        let code = match code {
            EXIT_SUCCESS | EXIT_FAILURE | EXIT_RESET => code,
            _ => exit_code_encode(code),
        };

        unsafe {
            asm!(
                "sw {0}, 0({1})",
                in(reg) code,
                in(reg) self.addr
            );
        }

        panic!("[kernel] Fail to shutdown.");
    }

    fn exit_success(&self) -> ! {
        self.exit(EXIT_SUCCESS);
    }

    fn exit_failure(&self) -> ! {
        self.exit(EXIT_FAILURE);
    }
}
