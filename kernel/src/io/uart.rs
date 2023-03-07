use bitflags::bitflags;
use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicPtr, Ordering},
};
use lazy_static::*;

const BS: u8 = 0x8;
const DEL: u8 = 0x7F;

macro_rules! wait_for {
    ($cond:expr) => {
        while !$cond {
            core::hint::spin_loop();
        }
    };
}

pub struct SerialPort {
    data_reg: AtomicPtr<u8>,       // receive holding reg & transmit holding reg
    int_en_reg: AtomicPtr<u8>,     // interrupt enable reg
    fifo_ctrl_reg: AtomicPtr<u8>,  // FIFO control reg
    line_ctrl_reg: AtomicPtr<u8>,  // line control reg
    modem_ctrl_reg: AtomicPtr<u8>, // unknown
    line_sts_reg: AtomicPtr<u8>,   // line status reg
}

bitflags! {
    struct IntEnFlags: u8 {
        const RECEIVED = 1;
        const SENT = 1 << 1;
        const ERRORED = 1 << 2;
        const STATUS_CHANGED = 1 << 3;
    }
}

bitflags! {
    struct LineStsFlags: u8 {
        const INPUT_FULL = 1;
        const OUTPUT_EMPTY = 1 << 5;
    }
}

impl SerialPort {
    /// Create a new UART interface on the given memory mapped address.
    ///
    /// This function is unsafe because caller must ensure that the given base address.
    pub unsafe fn new(base: usize) -> Self {
        let base_pointer = base as *mut u8;
        Self {
            data_reg: AtomicPtr::new(base_pointer),
            int_en_reg: AtomicPtr::new(base_pointer.add(1)),
            fifo_ctrl_reg: AtomicPtr::new(base_pointer.add(2)),
            line_ctrl_reg: AtomicPtr::new(base_pointer.add(3)),
            modem_ctrl_reg: AtomicPtr::new(base_pointer.add(4)),
            line_sts_reg: AtomicPtr::new(base_pointer.add(5)),
        }
    }

    /// Initialize the memory-mapped UART.
    ///
    /// Use the default configuration of 8-N-1.
    pub fn init(&self) {
        let self_data = self.data_reg.load(Ordering::Relaxed);
        let self_int_en = self.int_en_reg.load(Ordering::Relaxed);
        let self_fifo_ctrl = self.fifo_ctrl_reg.load(Ordering::Relaxed);
        let self_line_ctrl = self.line_ctrl_reg.load(Ordering::Relaxed);
        let self_modem_ctrl = self.modem_ctrl_reg.load(Ordering::Relaxed);

        unsafe {
            // disable interrupts
            self_int_en.write(0x00);

            // enable DLAB
            self_line_ctrl.write(0x80);

            // set maximum speed of 38.4K for LSB
            self_data.write(0x03);

            // set maximum speed of 38.4K for MSB
            self_int_en.write(0x00);

            // disable DLAB and set data word length to 8 bits
            self_line_ctrl.write(0x03);

            // enable FIFO, clear TX/RX queues and set interrupt watermark at 14 bytes
            self_fifo_ctrl.write(0xC7);

            // mark data terminal ready, signal request to send and enable auxilliary output
            self_modem_ctrl.write(0x0B);

            // enable interrupts
            self_int_en.write(0x01);
        }
    }

    /// Get line status.
    fn line_sts(&self) -> LineStsFlags {
        unsafe { LineStsFlags::from_bits_truncate(*self.line_sts_reg.load(Ordering::Relaxed)) }
    }

    /// Send a byte on the serial port.
    pub fn send(&self, data: u8) {
        let self_data_reg = self.data_reg.load(Ordering::Relaxed);
        unsafe {
            match data {
                BS | DEL => {
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self_data_reg.write(BS);
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self_data_reg.write(b' ');
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self_data_reg.write(BS);
                }

                _ => {
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self_data_reg.write(data);
                }
            }
        }
    }

    /// Receive a byte on the serial port.
    pub fn receive(&self) -> u8 {
        let self_data_reg = self.data_reg.load(Ordering::Relaxed);
        unsafe {
            wait_for!(self.line_sts().contains(LineStsFlags::INPUT_FULL));
            self_data_reg.read()
        }
    }
}

// TODO: decide whether to init in here or in the run time
// lazy_static! {
// pub static ref UART: SerialPort = unsafe { SerialPort::new(0x10_000_000) };
// }
pub static mut UART: MaybeUninit<SerialPort> = MaybeUninit::uninit();
