#![allow(dead_code)]
/// Almost everything in this files comes from NesDev: https://www.nesdev.org/wiki/CPU
use bitflags::bitflags;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Weak;

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
    fn run(&self, cpu: &mut CpuCore) -> u8;
    fn kind(&self) -> addr_modes::Kind;
}

trait Operation {
    // Some opcode requires additional clock cycles conditionally too
    fn run(&self, cpu: &mut CpuCore) -> u8;
}

struct Opcode {
    name: String,
    addr_mode: Box<dyn AddrMode>,
    op: Box<dyn Operation>,
    cycles: usize,
}

pub struct CpuCore {
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

    // Number of cycles left for the current instruction
    cycles: usize,
    // Total number of clock ticks from reset
    clock_count: usize,

    // Link to the underlying bus
    bus: Option<Weak<RefCell<Bus>>>,
}

impl CpuCore {
    fn new() -> Self {
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
            cycles: 0,
            clock_count: 0,
            bus: None,
        }
    }

    pub fn register_bus(&mut self, bus: Weak<RefCell<Bus>>) {
        self.bus = Some(bus)
    }

    fn read(&self, addr: u16) -> u8 {
        match &self.bus {
            None => panic!("CPU/read: No Bus"),
            Some(bus) => match bus.upgrade() {
                None => panic!("CPU/read: Bus has been dropped"),
                Some(bus) => bus.borrow().read(addr),
            },
        }
    }

    fn write(&self, addr: u16, value: u8) {
        match &self.bus {
            None => panic!("CPU/write: No Bus"),
            Some(bus) => match bus.upgrade() {
                None => panic!("CPU/write: Bus has been dropped"),
                Some(bus) => bus.borrow_mut().write(addr, value),
            },
        }
    }

    pub fn get_flag(&self, flag: Flags) -> bool {
        self.status.contains(flag)
    }

    pub fn set_flag(&mut self, flag: Flags, on_off: bool) {
        self.status.set(flag, on_off);
    }
}

pub struct Cpu {
    pub core: CpuCore,
    opcodes: HashMap<u8, Opcode>,
}

impl Cpu {
    pub fn new() -> Self {
        let opcodes = HashMap::new();

        // opcodes.insert(0, xxx);
        Self {
            core: CpuCore::new(),
            opcodes,
        }
    }

    pub fn clock(&mut self) {
        let Self { opcodes, core, .. } = self;

        if core.cycles == 0 {
            let opcode = core.read(core.pc);
            core.opcode = opcode;

            core.set_flag(Flags::U, true);

            core.pc += 1;

            let xxx = Opcode {
                name: "XXX".into(),
                addr_mode: Box::new(addr_modes::IMP {}),
                op: Box::new(operations::XXX {}),
                cycles: 0,
            };

            let Opcode {
                cycles,
                addr_mode,
                op,
                ..
            } = match opcodes.get(&opcode) {
                None => &xxx,
                Some(opcode) => opcode,
            };
            core.cycles = *cycles;

            let extra_cycle1 = addr_mode.run(core);
            let extra_cycle2 = op.run(core);

            core.cycles += (extra_cycle1 & extra_cycle2) as usize;

            // TODO:check if this is needed
            // core.set_flag(Flags::U, true);
        }

        core.cycles -= 1;
        core.clock_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_empty_flag() {
        let cpu = CpuCore::new();
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
        let mut cpu = CpuCore::new();
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
        let mut cpu = CpuCore::new();
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
