#![allow(unused_comparisons, dead_code)]

const RAM_SIZE: usize = 64 * 1024;

pub struct Bus {
    pub ram: [u8; RAM_SIZE],
}

impl Bus {
    pub fn new() -> Self {
        let ram = [0; RAM_SIZE];
        Self { ram }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        if addr >= 0x0000 && addr <= 0xFFFF {
            self.ram[addr as usize] = data
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        if addr >= 0x0000 && addr <= 0xFFFF {
            return self.ram[addr as usize];
        }
        return 0x00;
    }
}
