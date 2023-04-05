use alloc::sync::Arc;
use fosix::fs::FilePerm;
use spin::mutex::Mutex;

use crate::config::RING_BUFFER_SIZE;

#[derive(Clone)]
pub struct Pipe {
    perm: FilePerm,
    buf: Arc<Mutex<RingBuffer>>,
}

pub struct RingBuffer {
    buffer: [u8; RING_BUFFER_SIZE],
    head: usize,
    tail: usize,
}

impl Pipe {
    pub fn new() -> (Self, Self) {
        let ring_buf = Arc::new(Mutex::new(RingBuffer::new()));
        let pipe_read = Self {
            perm: FilePerm::READABLE,
            buf: ring_buf.clone(),
        };
        let pipe_write = Self {
            perm: FilePerm::WRITEABLE,
            buf: ring_buf.clone(),
        };
        (pipe_read, pipe_write)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut ring_buf = self.buf.lock();
        let mut bytes = 0;
        while bytes < buf.len() {
            if let Some(data) = ring_buf.pop() {
                buf[bytes] = data;
                bytes += 1;
            } else {
                break;
            }
        }
        bytes
    }

    pub fn write(&self, buf: &[u8]) -> usize {
        let mut ring_buf = self.buf.lock();
        let mut bytes = 0;
        while bytes < buf.len() {
            if ring_buf.push(buf[bytes]) {
                bytes += 1;
            } else {
                break;
            }
        }
        bytes
    }
}

impl RingBuffer {
    pub fn new() -> Self {
        Self {
            buffer: [0; RING_BUFFER_SIZE],
            head: 0,
            tail: 0,
        }
    }

    pub fn push(&mut self, data: u8) -> bool {
        if self.is_full() {
            return false;
        }
        self.buffer[self.tail] = data;
        self.tail = (self.tail + 1) % RING_BUFFER_SIZE;
        true
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }
        let data = self.buffer[self.head];
        self.head = (self.head + 1) % RING_BUFFER_SIZE;
        Some(data)
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    pub fn is_full(&self) -> bool {
        (self.tail + 1) % RING_BUFFER_SIZE == self.head
    }
}
