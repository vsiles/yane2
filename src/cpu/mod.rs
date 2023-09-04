#![allow(dead_code)]
/// Almost everything in this files comes from NesDev: https://www.nesdev.org/wiki/CPU
use bitflags::bitflags;
use std::collections::HashMap;

use crate::bus::Bus;

mod addr_modes;
mod operations;

bitflags! {
    pub struct Flags: u8 {
        const C = 1 << 0; // Carry Bit
        const Z = 1 << 1; // Zero
        const I = 1 << 2; // Disable Interrupts
        const D = 1 << 3; // Decimal Mode (not supported by Nes)
        const B = 1 << 4; // Break
        const U = 1 << 5; // Unused
        const V = 1 << 6; // Overflow
        const N = 1 << 7; // Negative
    }
}

trait AddrMode {
    // Addressing modes return 1 if additional clock cycles are necessary
    fn run(&self, cpu: &mut Cpu, bus: &Bus) -> u8;
    fn kind(&self) -> addr_modes::Kind;
}

trait Operation {
    // Some opcode requires additional clock cycles conditionally too
    fn run(&self, cpu: &mut Cpu, bus: &Bus) -> u8;
}

struct Opcode {
    name: String,
    addr_mode: Box<dyn AddrMode>,
    op: Box<dyn Operation>,
    cycles: usize,
}

pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    status: Flags,

    // temp variables to implement addressing modes + opcode logic
    fetched: u8,
    temp: u16,
    addr_abs: u16,
    addr_rel: u16,
    opcode: u8,

    opcodes: HashMap<u8, Opcode>,

    // Number of cycles left for the current instruction
    cycles: usize,
    // Total number of clock ticks from reset
    clock_count: usize,
}

impl Cpu {
    pub fn new() -> Self {
        let opcodes = HashMap::new();

        // opcodes.insert(0, xxx);

        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            pc: 0,
            status: Flags::empty(),
            fetched: 0,
            temp: 0,
            addr_abs: 0,
            addr_rel: 0,
            opcode: 0,
            opcodes,
            cycles: 0,
            clock_count: 0,
        }
    }

    pub fn get_flag(&self, flag: Flags) -> bool {
        self.status.contains(flag)
    }

    pub fn set_flag(&mut self, flag: Flags, on_off: bool) {
        self.status.set(flag, on_off);
    }

    // pub fn clock(&mut self, bus: &Bus) {
    //     if self.cycles == 0 {
    //         let opcode = bus.read(self.pc);
    //         self.opcode = opcode;

    //         self.set_flag(Flags::U, true);

    //         self.pc += 1;

    //         let xxx = Opcode {
    //             name: "XXX".into(),
    //             addr_mode: Box::new(addr_modes::IMP {}),
    //             op: Box::new(operations::XXX {}),
    //             cycles: 0
    //         };

    //         let Opcode{cycles, addr_mode, op, ..} =
    //             match self.opcodes.get(&opcode) {
    //                 None => &xxx,
    //                 Some(opcode) => opcode
    //             }
    //         ;
    //         self.cycles = *cycles;

    //         let extra_cycle1 = addr_mode.run(self, bus);
    //         let extra_cycle2 = op.run(self, bus);

    //         self.cycles += (extra_cycle1 & extra_cycle2) as usize;

    //         // TODO:check if this is needed
    //         // self.set_flag(Flags::U, true);
    //     }

    //     self.cycles -= 1;
    //     self.clock_count += 1;
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_empty_flag() {
        let cpu = Cpu::new();
        assert_eq!(cpu.get_flag(Flags::C), false);
        assert_eq!(cpu.get_flag(Flags::Z), false);
        assert_eq!(cpu.get_flag(Flags::I), false);
        assert_eq!(cpu.get_flag(Flags::D), false);
        assert_eq!(cpu.get_flag(Flags::B), false);
        assert_eq!(cpu.get_flag(Flags::U), false);
        assert_eq!(cpu.get_flag(Flags::V), false);
        assert_eq!(cpu.get_flag(Flags::N), false);
    }

    #[test]
    fn test_set_get_flags() {
        let mut cpu = Cpu::new();
        cpu.set_flag(Flags::C, true);
        cpu.set_flag(Flags::V, true);
        assert_eq!(cpu.get_flag(Flags::C), true);
        assert_eq!(cpu.get_flag(Flags::Z), false);
        assert_eq!(cpu.get_flag(Flags::I), false);
        assert_eq!(cpu.get_flag(Flags::D), false);
        assert_eq!(cpu.get_flag(Flags::B), false);
        assert_eq!(cpu.get_flag(Flags::U), false);
        assert_eq!(cpu.get_flag(Flags::V), true);
        assert_eq!(cpu.get_flag(Flags::N), false);
    }

    #[test]
    fn test_set_get_flags2() {
        let mut cpu = Cpu::new();
        cpu.set_flag(Flags::I | Flags::N, true);
        assert_eq!(cpu.get_flag(Flags::C), false);
        assert_eq!(cpu.get_flag(Flags::Z), false);
        assert_eq!(cpu.get_flag(Flags::I), true);
        assert_eq!(cpu.get_flag(Flags::D), false);
        assert_eq!(cpu.get_flag(Flags::B), false);
        assert_eq!(cpu.get_flag(Flags::U), false);
        assert_eq!(cpu.get_flag(Flags::V), false);
        assert_eq!(cpu.get_flag(Flags::N), true);
    }
}
