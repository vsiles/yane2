#![allow(unused_comparisons, dead_code)]
use crate::cpu::Cpu;

const RAM_SIZE: usize = 64 * 1024;

pub struct Bus {
    pub cpu: Cpu,
    pub ram: [u8; RAM_SIZE],
}

impl Bus {
    pub fn new() -> Self {
        let cpu = Cpu::new();
        let ram = [0; RAM_SIZE];
        Self { cpu, ram }
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
