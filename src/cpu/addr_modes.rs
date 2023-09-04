use super::{Cpu, AddrMode};
use super::Bus;

#[derive(PartialEq)]
pub enum Kind {
    IMP,
    IMM,
    ZP0,
    ZPX,
    ZPY,
    REL,
    ABS,
    ABX,
    ABY,
    IND,
    IZX,
    IZY,
}

pub struct IMP {}

impl AddrMode for IMP {
    fn run(&self, cpu: &mut Cpu, _bus: &Bus) -> u8 {
        cpu.fetched = cpu.a;
        0
    }

    fn kind(&self, ) -> Kind { Kind::IMP }
}

pub struct IMM {}
impl AddrMode for IMM {
    fn run(&self, cpu: &mut Cpu, _bus: &Bus) -> u8 {
        cpu.addr_abs = cpu.pc;
        cpu.pc += 1;
        0
    }
    fn kind(&self, ) -> Kind { Kind::IMM }
}

pub struct ZP0 {}
impl AddrMode for ZP0 {
    fn run(&self, cpu: &mut Cpu, bus: &Bus) -> u8 {
        cpu.addr_abs = bus.read(cpu.pc) as u16;
        cpu.pc += 1;
        0
    }
    fn kind(&self, ) -> Kind { Kind::ZP0 }
}

pub struct ZPX {}
impl AddrMode for ZPX {
    fn run(&self, cpu: &mut Cpu, bus: &Bus) -> u8 {
        cpu.addr_abs = bus.read(cpu.pc) as u16;
        cpu.addr_abs = cpu.addr_abs.overflowing_add(cpu.x as u16).0;
        cpu.addr_abs &= 0x00FF;
        cpu.pc += 1;
        0
    }
    fn kind(&self, ) -> Kind { Kind::ZPX }
}

pub struct ZPY {}
impl AddrMode for ZPY {
    fn run(&self, cpu: &mut Cpu, bus: &Bus) -> u8 {
        cpu.addr_abs = bus.read(cpu.pc) as u16;
        cpu.addr_abs = cpu.addr_abs.overflowing_add(cpu.y as u16).0;
        cpu.addr_abs &= 0x00FF;
        cpu.pc += 1;
        0
    }
    fn kind(&self, ) -> Kind { Kind::ZPY }
}

pub struct REL {}
impl AddrMode for REL {
    fn run(&self, cpu: &mut Cpu, bus: &Bus) -> u8 {
        cpu.addr_rel = bus.read(cpu.pc) as u16;
        cpu.pc += 1;
        if cpu.addr_rel & 0x0080 != 0 {
            // relative range between -128 and +127 so we sign extend
            cpu.addr_rel |= 0xFF00;
        }
        0
    }
    fn kind(&self, ) -> Kind { Kind::REL }
}

pub struct ABS {}
impl AddrMode for ABS {
    fn run(&self, cpu: &mut Cpu, bus: &Bus) -> u8 {
        let low : u16 = bus.read(cpu.pc) as u16;
        cpu.pc += 1;
        let high : u16 = bus.read(cpu.pc) as u16;
        cpu.pc += 1;

        cpu.addr_abs = (high << 8) | low;
        0
    }
    fn kind(&self, ) -> Kind { Kind::ABS }
}

pub struct ABX {}
impl AddrMode for ABX {
    fn run(&self, cpu: &mut Cpu, bus: &Bus) -> u8 {
        let low : u16 = bus.read(cpu.pc) as u16;
        cpu.pc += 1;
        let high : u16 = bus.read(cpu.pc) as u16;
        cpu.pc += 1;

        cpu.addr_abs = (high << 8) | low;
        cpu.addr_abs = cpu.addr_abs.overflowing_add(cpu.x as u16).0;

        // maybe an extra clock cycle is necessary
        let extra_clock_cycle = (cpu.addr_abs & 0xFF00) != (high << 8);
        extra_clock_cycle as u8
    }
    fn kind(&self, ) -> Kind { Kind::ABX }
}

pub struct ABY {}
impl AddrMode for ABY {
    fn run(&self, cpu: &mut Cpu, bus: &Bus) -> u8 {
        let low : u16 = bus.read(cpu.pc) as u16;
        cpu.pc += 1;
        let high : u16 = bus.read(cpu.pc) as u16;
        cpu.pc += 1;

        cpu.addr_abs = (high << 8) | low;
        cpu.addr_abs = cpu.addr_abs.overflowing_add(cpu.y as u16).0;

        // maybe an extra clock cycle is necessary
        let extra_clock_cycle = (cpu.addr_abs & 0xFF00) != (high << 8);
        extra_clock_cycle as u8
    }
    fn kind(&self, ) -> Kind { Kind::ABY }
}

pub struct IND {}
impl AddrMode for IND {
    fn run(&self, cpu: &mut Cpu, bus: &Bus) -> u8 {
        let ptr_low : u16 = bus.read(cpu.pc) as u16;
        cpu.pc += 1;
        let ptr_high : u16 = bus.read(cpu.pc) as u16;
        cpu.pc += 1;

        let ptr : u16 = (ptr_high << 8) | ptr_low;

        if ptr_low == 0x00FF {
            // page boundary hardware bug
            let low : u16 = bus.read(ptr) as u16;
            let high: u16 = bus.read(ptr & 0xFF00) as u16;
            cpu.addr_abs = (high << 8) | low
        } else {
            let low : u16 = bus.read(ptr) as u16;
            let high: u16 = bus.read(ptr + 1) as u16;
            cpu.addr_abs = (high << 8) | low
        }
        0
    }
    fn kind(&self, ) -> Kind { Kind::IND }
}

pub struct IZX {}
impl AddrMode for IZX {
    fn run(&self, cpu: &mut Cpu, bus: &Bus) -> u8 {
        let ptr : u16 = bus.read(cpu.pc) as u16;
        cpu.pc += 1;

        // ptr is really 8 bits, and x too, so ptr + x + 1 can't overflow in u16
        let ptr_x : u16 = ptr + (cpu.x as u16);
        let low: u16 = bus.read(ptr_x) as u16;
        let high: u16 = bus.read(ptr_x + 1) as u16;

        cpu.addr_abs = (high << 8) | low;
        0
    }
    fn kind(&self, ) -> Kind { Kind::IZX }
}

pub struct IZY {}
impl AddrMode for IZY {
    fn run(&self, cpu: &mut Cpu, bus: &Bus) -> u8 {
        let ptr : u16 = bus.read(cpu.pc) as u16;
        cpu.pc += 1;

        // ptr is really 8 bits, so ptr + x + 1 can't overflow in u16
        let low: u16 = bus.read(ptr) as u16;
        let high: u16 = bus.read(ptr + 1) as u16;

        cpu.addr_abs = (high << 8) | low;
        cpu.addr_abs = cpu.addr_abs.overflowing_add(cpu.y as u16).0;

        let extra_cycle = (cpu.addr_abs & 0xFF00) != (high << 8);
        return extra_cycle as u8
    }
    fn kind(&self, ) -> Kind { Kind::IZY }
}
