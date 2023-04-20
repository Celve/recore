use alloc::collections::VecDeque;
use bitflags::bitflags;
use spin::Spin;
use volatile::{ReadOnly, Volatile};

macro_rules! wait_for {
    ($cond:expr) => {
        while !$cond {
            core::hint::spin_loop();
        }
    };
}

const BS: u8 = 0x8;
const DEL: u8 = 0x7F;

/// Read port when DLAB = 0.
pub struct ReadPort {
    /// receive buffer
    rbr: Volatile<u8>,
    /// interrupt enable
    ier: Volatile<InterruptEnable>,
    /// interrupt identification
    iir: ReadOnly<u8>,
    /// line control
    lcr: Volatile<LineControl>,
    /// modem control
    mcr: Volatile<ModemControl>,
    /// line status
    lsr: ReadOnly<LineStatus>,
    /// modem status
    msr: ReadOnly<u8>,
    // scratch
    scr: ReadOnly<u8>,
}

/// Write port when DLAB = 0.
pub struct WritePort {
    /// transmitter holding
    thr: Volatile<u8>,
    /// interrupt enable
    ier: Volatile<InterruptEnable>,
    /// FIFO control
    fcr: Volatile<FifoControl>,
    /// line control
    lcr: Volatile<LineControl>,
    /// modem control
    mcr: Volatile<ModemControl>,
    /// line status
    lsr: ReadOnly<LineStatus>,
    /// not used
    _padding: ReadOnly<u8>,
    // scratch
    scr: ReadOnly<u8>,
}

bitflags! {
    pub struct InterruptEnable: u8 {
        const RX_AVAILABLE = 1 << 0;
        const TX_EMPTY = 1 << 1;
    }

    pub struct FifoControl: u8 {
        const ENABLE = 1 << 0;
        const CLEAR_RX_FIFO = 1 << 1;
        const CLEAR_TX_FIFO = 1 << 2;
        const TRIGGER_14 = 0b11 << 6;
    }

    pub struct LineControl: u8 {
        const DATA_8 = 0b11;
        const DLAB_ENABLE = 1 << 7;
    }

    pub struct ModemControl: u8 {
        const DATA_TERMINAL_READY = 1 << 0;
        const AUXILIARY_OUTPUT_2 = 1 << 3;
    }

    pub struct LineStatus: u8 {
        const INPUT_AVAILABLE = 1 << 0;
        const OUTPUT_EMPTY = 1 << 5;
    }
}

/// This is a serial UART, which stands for universal asynchronous receiver/transmitter.
///
/// The UART that QEMU implemented follows the hardware standare of NS16550A.
/// Hence, this structure provides some interfaces that help us manage the UART.
pub struct UartRaw {
    base: usize,
}

pub struct UartInner {
    uart: UartRaw,
    read_buffer: VecDeque<u8>,
}

pub struct Uart {
    inner: Spin<UartInner>,
}

impl UartRaw {
    pub fn new(base: usize) -> Self {
        Self { base }
    }

    fn read_port(&self) -> &'static mut ReadPort {
        unsafe { &mut *(self.base as *mut ReadPort) }
    }

    fn write_port(&self) -> &'static mut WritePort {
        unsafe { &mut *(self.base as *mut WritePort) }
    }

    pub fn init(&self) {
        let read_port = self.read_port();
        let write_port = self.write_port();

        // disable interrupts
        read_port.ier.write(InterruptEnable::empty());

        // enable DLAB
        read_port.lcr.write(LineControl::DLAB_ENABLE);

        // set maximum speed of 38.4K for LSB
        read_port.rbr.write(0x03);

        // set maximum speed of 38.4K for MSB
        read_port.ier.write(InterruptEnable::empty()); // namely 0

        // disable DLAB and set data word length to 8 bits
        read_port.lcr.write(LineControl::DATA_8);

        // enable FIFO, clear TX/RX queues and set interrupt watermark at 14 bytes
        write_port
            .fcr
            .write(FifoControl::ENABLE | FifoControl::TRIGGER_14);

        // mark data terminal ready, signal request to send and enable auxilliary output
        read_port
            .mcr
            .write(ModemControl::DATA_TERMINAL_READY | ModemControl::AUXILIARY_OUTPUT_2);

        // enable interrupts
        read_port
            .ier
            .write(InterruptEnable::RX_AVAILABLE | InterruptEnable::TX_EMPTY);
    }

    /// Send a byte on the serial port.
    pub fn send(&self, data: u8) {
        let write_port = self.write_port();
        let lsr = &write_port.lsr;
        let thr = &mut write_port.thr;
        match data {
            BS | DEL => {
                wait_for!(lsr.read().contains(LineStatus::OUTPUT_EMPTY));
                thr.write(BS);
                wait_for!(lsr.read().contains(LineStatus::OUTPUT_EMPTY));
                thr.write(b' ');
                wait_for!(lsr.read().contains(LineStatus::OUTPUT_EMPTY));
                thr.write(BS);
            }
            _ => {
                wait_for!(lsr.read().contains(LineStatus::OUTPUT_EMPTY));
                thr.write(data);
            }
        }
    }

    pub fn recv(&self) -> Option<u8> {
        let read_port = self.read_port();
        let lsr = &read_port.lsr;
        let rbr = &read_port.rbr;
        if lsr.read().contains(LineStatus::INPUT_AVAILABLE) {
            Some(rbr.read())
        } else {
            None
        }
    }
}

impl UartInner {
    pub fn new(base: usize) -> Self {
        Self {
            uart: UartRaw::new(base),
            read_buffer: VecDeque::new(),
        }
    }
}
