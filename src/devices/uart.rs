use std::io::{self, Write};

pub struct Uart;

impl Uart {
    pub fn new() -> Self {
        Self
    }
    pub fn read_u8(&self, _addr: u64) -> u8 {
        0
    }
    pub fn write_u8(&mut self, _addr: u64, val: u8) {
        print!("{}", val as char);
        io::stdout().flush().ok();
    }
}
