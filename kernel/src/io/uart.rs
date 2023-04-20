use bitflags::bitflags;
use core::sync::atomic::{AtomicPtr, Ordering};
use lazy_static::lazy_static;
use spin::Spin;

use crate::{config::UART_BASE_ADDRESS, drivers::uart::UartRaw, task::processor::fetch_curr_task};

const BS: u8 = 0x8;
const DEL: u8 = 0x7F;
const IER_RX_ENABLE: u8 = 1 << 0;
const IER_TX_ENABLE: u8 = 1 << 1;

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
            self_int_en.write(IER_RX_ENABLE | IER_TX_ENABLE);
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
    pub fn recv(&self) -> u8 {
        let self_data_reg = self.data_reg.load(Ordering::Relaxed);
        unsafe {
            wait_for!(self.line_sts().contains(LineStsFlags::INPUT_FULL));
            self_data_reg.read()
        }
    }

    pub fn try_recv(&self) -> Option<u8> {
        let self_data_reg = self.data_reg.load(Ordering::Relaxed);
        if self.line_sts().contains(LineStsFlags::INPUT_FULL) {
            Some(unsafe { self_data_reg.read() })
        } else {
            None
        }
    }
}

struct UartTx;

struct UartRx;

impl UartTx {
    pub fn send(&self, data: u8) {
        UART.send(data);
    }
}

impl UartRx {
    // pub fn recv(&self) -> u8 {
    // UART.recv()
    // }

    pub fn try_recv(&self) -> Option<u8> {
        UART.recv()
    }
}

lazy_static! {
    // static ref UART: SerialPort = unsafe { SerialPort::new(UART_BASE_ADDRESS) };
    static ref UART: UartRaw = unsafe { UartRaw::new(UART_BASE_ADDRESS) };
    static ref UART_TX: UartTx = UartTx;
    static ref UART_RX: Spin<UartRx> = Spin::new(UartRx);
}

pub fn send_to_uart(data: u8) {
    UART_TX.send(data);
}

#[no_mangle]
pub fn recv_from_uart() -> u8 {
    loop {
        if let Some(uart_rx) = UART_RX.try_lock() {
            loop {
                if let Some(c) = uart_rx.try_recv() {
                    return c;
                } else {
                    fetch_curr_task().yield_now();
                }
            }
        } else {
            fetch_curr_task().yield_now();
        }
    }
}

pub fn init_uart() {
    UART.init();
}
