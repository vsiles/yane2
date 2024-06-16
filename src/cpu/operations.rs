#![allow(clippy::upper_case_acronyms)]
use super::{CpuCore, Flags, Opcode, Operation};
use std::collections::HashMap;

pub struct XXX {}

impl Operation for XXX {
    fn run(&self, _: &HashMap<u8, Opcode>, _: &mut CpuCore) -> u8 {
        0
    }
}

pub struct LDA {}

impl Operation for LDA {
    fn run(&self, opcodes: &HashMap<u8, Opcode>, cpu: &mut CpuCore) -> u8 {
        let a = cpu.fetch(opcodes);
        cpu.a = a;
        cpu.set_flag(Flags::Z, a == 0x00);
        cpu.set_flag(Flags::N, (a & 0x80) != 0);
        1
    }
}

pub struct LDX {}

impl Operation for LDX {
    fn run(&self, opcodes: &HashMap<u8, Opcode>, cpu: &mut CpuCore) -> u8 {
        let x = cpu.fetch(opcodes);
        cpu.x = x;
        cpu.set_flag(Flags::Z, x == 0x00);
        cpu.set_flag(Flags::N, (x & 0x80) != 0);
        1
    }
}

pub struct LDY {}

impl Operation for LDY {
    fn run(&self, opcodes: &HashMap<u8, Opcode>, cpu: &mut CpuCore) -> u8 {
        let y = cpu.fetch(opcodes);
        cpu.y = y;
        cpu.set_flag(Flags::Z, y == 0x00);
        cpu.set_flag(Flags::N, (y & 0x80) != 0);
        1
    }
}

pub struct STA {}

impl Operation for STA {
    fn run(&self, _opcodes: &HashMap<u8, Opcode>, cpu: &mut CpuCore) -> u8 {
        cpu.write(cpu.addr_abs, cpu.a);
        0
    }
}

pub struct STX {}

impl Operation for STX {
    fn run(&self, _opcodes: &HashMap<u8, Opcode>, cpu: &mut CpuCore) -> u8 {
        cpu.write(cpu.addr_abs, cpu.x);
        0
    }
}

pub struct STY {}

impl Operation for STY {
    fn run(&self, _opcodes: &HashMap<u8, Opcode>, cpu: &mut CpuCore) -> u8 {
        cpu.write(cpu.addr_abs, cpu.y);
        0
    }
}

pub struct CLC {}

impl Operation for CLC {
    fn run(&self, _opcodes: &HashMap<u8, Opcode>, cpu: &mut CpuCore) -> u8 {
        cpu.set_flag(Flags::C, false);
        0
    }
}

pub struct ADC {}

impl Operation for ADC {
    fn run(&self, opcodes: &HashMap<u8, Opcode>, cpu: &mut CpuCore) -> u8 {
        let fetched = cpu.fetch(opcodes) as u16;

        // working in u16 to catch overflow more easily
        let a = cpu.a as u16;
        let c = cpu.get_flag(Flags::C) as u16;

        let temp = a + fetched + c;

        cpu.set_flag(Flags::C, temp > 255);
        cpu.set_flag(Flags::Z, (temp & 0x00FF) == 0);
        let v = !(a ^ fetched) & (a ^ temp);
        cpu.set_flag(Flags::V, (v & 0x0080) != 0);
        cpu.set_flag(Flags::N, (temp & 0x0080) != 0);

        cpu.a = (temp & 0x00FF) as u8;
        1
    }
}

pub struct DEX {}

impl Operation for DEX {
    fn run(&self, _opcodes: &HashMap<u8, Opcode>, cpu: &mut CpuCore) -> u8 {
        cpu.x = cpu.x.wrapping_sub(1);
        cpu.set_flag(Flags::Z, cpu.x == 0x00);
        cpu.set_flag(Flags::N, (cpu.x & 0x80) != 0);
        0
    }
}

pub struct DEY {}

impl Operation for DEY {
    fn run(&self, _opcodes: &HashMap<u8, Opcode>, cpu: &mut CpuCore) -> u8 {
        cpu.y = cpu.y.wrapping_sub(1);
        cpu.set_flag(Flags::Z, cpu.y == 0x00);
        cpu.set_flag(Flags::N, (cpu.y & 0x80) != 0);
        0
    }
}

pub struct BNE {}

impl Operation for BNE {
    fn run(&self, _opcodes: &HashMap<u8, Opcode>, cpu: &mut CpuCore) -> u8 {
        if !cpu.get_flag(Flags::Z) {
            cpu.cycles += 1;
            cpu.addr_abs = cpu.pc.wrapping_add(cpu.addr_rel);

            if (cpu.addr_abs & 0xFF00) != (cpu.pc & 0xFF00) {
                cpu.cycles += 1
            }

            cpu.pc = cpu.addr_abs
        }
        0
    }
}

pub struct NOP {}

impl Operation for NOP {
    fn run(&self, _opcodes: &HashMap<u8, Opcode>, cpu: &mut CpuCore) -> u8 {
        match cpu.opcode {
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => 1,
            _ => 0,
        }
    }
}
