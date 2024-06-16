#![allow(dead_code)]
/// Almost everything in this files comes from NesDev: https://www.nesdev.org/wiki/CPU
use bitflags::bitflags;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::RwLock;

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
    fn run(&self, opcodse: &HashMap<u8, Opcode>, cpu: &mut CpuCore) -> u8;
}

struct Opcode {
    name: String,
    addr_mode: Box<dyn AddrMode>,
    op: Box<dyn Operation>,
    cycles: usize,
}

macro_rules! opcode {
    ($name:ident, $mode: ident, $cycles: expr) => {
        Opcode {
            name: stringify!($name).into(),
            addr_mode: Box::new(addr_modes::$mode {}),
            op: Box::new(operations::$name {}),
            cycles: $cycles,
        }
    };
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
    bus: Rc<RwLock<Bus>>,
}

impl CpuCore {
    fn new(bus: Bus) -> Self {
        let bus = Rc::new(RwLock::new(bus));
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
            bus,
        }
    }

    fn read(&self, addr: u16) -> u8 {
        self.bus.read().expect("Failed to get bus").read(addr)
    }

    fn write(&self, addr: u16, value: u8) {
        self.bus
            .write()
            .expect("Failed to get bus")
            .write(addr, value)
    }

    pub fn get_flag(&self, flag: Flags) -> bool {
        self.status.contains(flag)
    }

    pub fn set_flag(&mut self, flag: Flags, on_off: bool) {
        self.status.set(flag, on_off);
    }

    fn fetch(&mut self, opcodes: &HashMap<u8, Opcode>) -> u8 {
        let xxx = opcode!(XXX, IMP, 0);

        let Opcode { addr_mode, .. } = match opcodes.get(&self.opcode) {
            None => &xxx,
            Some(opcode) => opcode,
        };
        match addr_mode.kind() {
            addr_modes::Kind::IMP => {}
            _ => self.fetched = self.read(self.addr_abs),
        }
        self.fetched
    }

    fn reset(&mut self) {
        self.addr_abs = 0xFFFC;
        let low = self.read(self.addr_abs) as u16;
        let high = self.read(self.addr_abs + 1) as u16;

        self.pc = (high << 8) | low;

        self.a = 0x00;
        self.x = 0x00;
        self.y = 0x00;
        self.sp = 0xFD;
        self.status = Flags::U;

        self.addr_rel = 0x0000;
        self.addr_abs = 0x0000;
        self.fetched = 0x00;

        self.cycles = 8;
    }

    fn complete(&self) -> bool {
        self.cycles == 0
    }
}

pub struct Cpu {
    pub core: CpuCore,
    opcodes: HashMap<u8, Opcode>,
}

macro_rules! add_opcode {
    ($opcodes:ident, $ndx: expr, $opcode: expr) => {
        $opcodes.insert($ndx, $opcode)
    };
}

impl Cpu {
    pub fn bus(&self) -> Rc<RwLock<Bus>> {
        self.core.bus.clone()
    }

    pub fn new(bus: Bus) -> Self {
        let mut opcodes = HashMap::new();

        /* opcode info mostly comes from
           https://www.nesdev.org/wiki/Visual6502wiki/6502_all_256_Opcodes
        */

        add_opcode!(opcodes, 0x04, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x0C, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x14, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x1A, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x1C, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x34, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x3A, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x3C, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x44, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x54, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x5A, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x5C, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x64, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x74, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x7A, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x7C, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x80, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x82, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0x89, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0xC2, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0xD4, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0xDA, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0xDC, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0xE2, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0xEA, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0xF4, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0xFA, opcode!(NOP, IMP, 0));
        add_opcode!(opcodes, 0xFC, opcode!(NOP, IMP, 0));

        add_opcode!(opcodes, 0xA1, opcode!(LDA, IZX, 6));
        add_opcode!(opcodes, 0xA5, opcode!(LDA, ZP0, 3));
        add_opcode!(opcodes, 0xA9, opcode!(LDA, IMM, 2));
        add_opcode!(opcodes, 0xAD, opcode!(LDA, ABS, 4));
        add_opcode!(opcodes, 0xB1, opcode!(LDA, IZY, 5));
        add_opcode!(opcodes, 0xB5, opcode!(LDA, ZPX, 4));
        add_opcode!(opcodes, 0xB9, opcode!(LDA, ABY, 4));
        add_opcode!(opcodes, 0xBD, opcode!(LDA, ABX, 4));

        add_opcode!(opcodes, 0xA2, opcode!(LDX, IMM, 2));
        add_opcode!(opcodes, 0xA6, opcode!(LDX, ZP0, 3));
        add_opcode!(opcodes, 0xAE, opcode!(LDX, ABS, 4));
        add_opcode!(opcodes, 0xB6, opcode!(LDX, ZPY, 4));
        add_opcode!(opcodes, 0xBE, opcode!(LDX, ABY, 4));

        add_opcode!(opcodes, 0xA0, opcode!(LDY, IMM, 2));
        add_opcode!(opcodes, 0xA4, opcode!(LDY, ZP0, 3));
        add_opcode!(opcodes, 0xAC, opcode!(LDY, ABS, 4));
        add_opcode!(opcodes, 0xB4, opcode!(LDY, ZPX, 4));
        add_opcode!(opcodes, 0xBC, opcode!(LDY, ABX, 4));

        add_opcode!(opcodes, 0x81, opcode!(STA, IZX, 6));
        add_opcode!(opcodes, 0x85, opcode!(STA, ZP0, 3));
        add_opcode!(opcodes, 0x8D, opcode!(STA, ABS, 4));
        add_opcode!(opcodes, 0x91, opcode!(STA, IZY, 6));
        add_opcode!(opcodes, 0x95, opcode!(STA, ZPX, 4));
        add_opcode!(opcodes, 0x99, opcode!(STA, ABY, 5));
        add_opcode!(opcodes, 0x9D, opcode!(STA, ABX, 5));

        add_opcode!(opcodes, 0x86, opcode!(STX, ZP0, 3));
        add_opcode!(opcodes, 0x8E, opcode!(STX, ABS, 4));
        add_opcode!(opcodes, 0x96, opcode!(STX, ZPY, 4));

        add_opcode!(opcodes, 0x84, opcode!(STY, ZP0, 3));
        add_opcode!(opcodes, 0x8C, opcode!(STY, ABS, 4));
        add_opcode!(opcodes, 0x94, opcode!(STY, ZPX, 4));

        add_opcode!(opcodes, 0x61, opcode!(ADC, IZX, 6));
        add_opcode!(opcodes, 0x65, opcode!(ADC, ZP0, 3));
        add_opcode!(opcodes, 0x69, opcode!(ADC, IMM, 2));
        add_opcode!(opcodes, 0x6D, opcode!(ADC, ABS, 4));
        add_opcode!(opcodes, 0x71, opcode!(ADC, IZY, 5));
        add_opcode!(opcodes, 0x75, opcode!(ADC, ZPX, 4));
        add_opcode!(opcodes, 0x79, opcode!(ADC, ABY, 4));
        add_opcode!(opcodes, 0x7D, opcode!(ADC, ABX, 4));

        add_opcode!(opcodes, 0x18, opcode!(CLC, IMP, 2));

        add_opcode!(opcodes, 0xCA, opcode!(DEX, IMP, 2));
        add_opcode!(opcodes, 0x88, opcode!(DEY, IMP, 2));

        add_opcode!(opcodes, 0xD0, opcode!(BNE, REL, 3));

        Self {
            core: CpuCore::new(bus),
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

            let xxx = opcode!(XXX, IMP, 0);

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
            let extra_cycle2 = op.run(opcodes, core);

            core.cycles += (extra_cycle1 & extra_cycle2) as usize;

            // TODO:check if this is needed
            // core.set_flag(Flags::U, true);
        }

        core.cycles -= 1;
        core.clock_count += 1;
    }

    pub fn reset(&mut self) {
        self.core.reset()
    }

    pub fn complete(&self) -> bool {
        self.core.complete()
    }

    pub fn disassemble(&self, start_addr: u16, stop_addr: u16) -> BTreeMap<u16, String> {
        let mut addr = start_addr;
        let mut lines = BTreeMap::new();

        let xxx = opcode!(XXX, IMP, 0);

        while addr <= stop_addr {
            let line_addr = addr;

            let mut line = format!("${:>04X}: ", addr);
            let opcode = self.core.read(addr);

            let Opcode {
                name, addr_mode, ..
            } = match self.opcodes.get(&opcode) {
                None => &xxx,
                Some(opcode) => opcode,
            };

            if addr == 0xFFFF {
                break;
            }
            addr += 1;
            line = format!("{}{} ", line, name);

            match addr_mode.kind() {
                addr_modes::Kind::IMP => line = format!("{} {{IMP}}", line),
                addr_modes::Kind::IMM => {
                    let value = self.core.read(addr);
                    addr += 1;
                    line = format!("{}#${:>02X} {{IMM}}", line, value)
                }
                addr_modes::Kind::ZP0 => {
                    let low = self.core.read(addr);
                    addr += 1;
                    line = format!("{}${:>02X} {{ZP0}}", line, low)
                }
                addr_modes::Kind::ZPX => {
                    let low = self.core.read(addr);
                    addr += 1;
                    line = format!("{}${:>02X}, X {{ZPX}}", line, low)
                }
                addr_modes::Kind::ZPY => {
                    let low = self.core.read(addr);
                    addr += 1;
                    line = format!("{}${:>02X}, Y {{ZPY}}", line, low)
                }
                addr_modes::Kind::IZX => {
                    let low = self.core.read(addr);
                    addr += 1;
                    line = format!("{}(${:>02X}, X) {{IZX}}", line, low)
                }
                addr_modes::Kind::IZY => {
                    let low = self.core.read(addr);
                    addr += 1;
                    line = format!("{}(${:>02X}), Y {{IZY}}", line, low)
                }
                addr_modes::Kind::ABS => {
                    let low = self.core.read(addr) as u16;
                    addr += 1;
                    let high = self.core.read(addr) as u16;
                    addr += 1;
                    line = format!("{}${:>04X} {{ABS}}", line, (high << 8) | low)
                }
                addr_modes::Kind::ABX => {
                    let low = self.core.read(addr) as u16;
                    addr += 1;
                    let high = self.core.read(addr) as u16;
                    addr += 1;
                    line = format!("{}${:>04X}, X {{ABX}}", line, (high << 8) | low)
                }
                addr_modes::Kind::ABY => {
                    let low = self.core.read(addr) as u16;
                    addr += 1;
                    let high = self.core.read(addr) as u16;
                    addr += 1;
                    line = format!("{}${:>04X}, Y {{ABY}}", line, (high << 8) | low)
                }
                addr_modes::Kind::IND => {
                    let low = self.core.read(addr) as u16;
                    addr += 1;
                    let high = self.core.read(addr) as u16;
                    addr += 1;
                    line = format!("{}(${:>04X}) {{IND}}", line, (high << 8) | low)
                }
                addr_modes::Kind::REL => {
                    let value = self.core.read(addr);
                    addr += 1;
                    line = format!(
                        "{}${:>02X} [${:>04X}] {{REL}}",
                        line,
                        value,
                        addr + value as u16
                    )
                }
            }
            lines.insert(line_addr, line);
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_empty_flag() {
        let cpu = CpuCore::new(Bus::new());
        assert!(!cpu.get_flag(Flags::C));
        assert!(!cpu.get_flag(Flags::Z));
        assert!(!cpu.get_flag(Flags::I));
        assert!(!cpu.get_flag(Flags::D));
        assert!(!cpu.get_flag(Flags::B));
        assert!(!cpu.get_flag(Flags::U));
        assert!(!cpu.get_flag(Flags::V));
        assert!(!cpu.get_flag(Flags::N));
    }

    #[test]
    fn test_set_get_flags() {
        let mut cpu = CpuCore::new(Bus::new());
        cpu.set_flag(Flags::C, true);
        cpu.set_flag(Flags::V, true);
        assert!(cpu.get_flag(Flags::C));
        assert!(!cpu.get_flag(Flags::Z));
        assert!(!cpu.get_flag(Flags::I));
        assert!(!cpu.get_flag(Flags::D));
        assert!(!cpu.get_flag(Flags::B));
        assert!(!cpu.get_flag(Flags::U));
        assert!(cpu.get_flag(Flags::V));
        assert!(!cpu.get_flag(Flags::N));
    }

    #[test]
    fn test_set_get_flags2() {
        let mut cpu = CpuCore::new(Bus::new());
        cpu.set_flag(Flags::I | Flags::N, true);
        assert!(!cpu.get_flag(Flags::C));
        assert!(!cpu.get_flag(Flags::Z));
        assert!(cpu.get_flag(Flags::I));
        assert!(!cpu.get_flag(Flags::D));
        assert!(!cpu.get_flag(Flags::B));
        assert!(!cpu.get_flag(Flags::U));
        assert!(!cpu.get_flag(Flags::V));
        assert!(cpu.get_flag(Flags::N));
    }
}

// Reference

/*
 00 BRK 7        $00: bytes: 0 cycles: 0 _____=>_____ __
 01 ORA izx 6    $01: bytes: 2 cycles: 6 A____=>____P R_ izx
 02 *KIL         $02: CRASH
 03 *SLO izx 8   $03: bytes: 2 cycles: 8 A____=>____P RW izx
 04 *NOP zp 3    $04: bytes: 2 cycles: 3 _____=>_____ R_ zp
 05 ORA zp 3     $05: bytes: 2 cycles: 3 A____=>A___P R_ zp
 06 ASL zp 5     $06: bytes: 2 cycles: 5 _____=>____P RW zp
 07 *SLO zp 5    $07: bytes: 2 cycles: 5 A____=>A___P RW zp
 08 PHP 3        $08: bytes: 1 cycles: 3 ___SP=>___S_ _W
 09 ORA imm 2    $09: bytes: 2 cycles: 2 _____=>A___P __
 0A ASL 2        $0A: bytes: 1 cycles: 2 A____=>A___P __
 0B *ANC imm 2   $0B: bytes: 2 cycles: 2 A____=>____P __
 0C *NOP abs 4   $0C: bytes: 3 cycles: 4 _____=>_____ R_ abs
 0D ORA abs 4    $0D: bytes: 3 cycles: 4 A____=>A___P R_ abs
 0E ASL abs 6    $0E: bytes: 3 cycles: 6 _____=>____P RW abs
 0F *SLO abs 6   $0F: bytes: 3 cycles: 6 A____=>A___P RW abs
 10 BPL rel 2*   $10: bytes: 2 cycles: 3 ____P=>_____ __
 11 ORA izy 5*   $11: bytes: 2 cycles: 5 A____=>____P R_ izy
 12 *KIL         $12: CRASH
 13 *SLO izy 8   $13: bytes: 2 cycles: 8 A____=>____P RW izy
 14 *NOP zpx 4   $14: bytes: 2 cycles: 4 _____=>_____ R_ zpx
 15 ORA zpx 4    $15: bytes: 2 cycles: 4 A____=>A___P R_ zpx
 16 ASL zpx 6    $16: bytes: 2 cycles: 6 _____=>____P RW zpx
 17 *SLO zpx 6   $17: bytes: 2 cycles: 6 A____=>A___P RW zpx
 18 CLC 2        $18: bytes: 1 cycles: 2 _____=>____P __
 19 ORA aby 4*   $19: bytes: 3 cycles: 4 A____=>A___P R_ absy
 1A *NOP 2       $1A: bytes: 1 cycles: 2 _____=>_____ __
 1B *SLO aby 7   $1B: bytes: 3 cycles: 7 A____=>A___P RW absy
 1C *NOP abx 4*  $1C: bytes: 3 cycles: 4 _____=>_____ R_ absx
 1D ORA abx 4*   $1D: bytes: 3 cycles: 4 A____=>A___P R_ absx
 1E ASL abx 7    $1E: bytes: 3 cycles: 7 _____=>____P RW absx
 1F *SLO abx 7   $1F: bytes: 3 cycles: 7 A____=>A___P RW absx
 20 JSR abs 6    $20: bytes: X cycles: 6 ___S_=>___S_ _W
 21 AND izx 6    $21: bytes: 2 cycles: 6 _____=>A___P R_ izx
 22 *KIL         $22: CRASH
 23 *RLA izx 8   $23: bytes: 2 cycles: 8 ____P=>A___P RW izx
 24 BIT zp 3     $24: bytes: 2 cycles: 3 A____=>____P R_ zp
 25 AND zp 3     $25: bytes: 2 cycles: 3 A____=>A___P R_ zp
 26 ROL zp 5     $26: bytes: 2 cycles: 5 ____P=>____P RW zp
 27 *RLA zp 5    $27: bytes: 2 cycles: 5 A___P=>A___P RW zp
 28 PLP 4        $28: bytes: 1 cycles: 4 ___S_=>___SP __
 29 AND imm 2    $29: bytes: 2 cycles: 2 A____=>A___P __
 2A ROL 2        $2A: bytes: 1 cycles: 2 A___P=>A___P __
 2B *ANC imm 2   $2B: bytes: 2 cycles: 2 A____=>____P __
 2C BIT abs 4    $2C: bytes: 3 cycles: 4 A____=>____P R_ abs
 2D AND abs 4    $2D: bytes: 3 cycles: 4 A____=>A___P R_ abs
 2E ROL abs 6    $2E: bytes: 3 cycles: 6 ____P=>____P RW abs
 2F *RLA abs 6   $2F: bytes: 3 cycles: 6 A___P=>A___P RW abs
 30 BMI rel 2*   $30: bytes: 2 cycles: 2 _____=>_____ __
 31 AND izy 5*   $31: bytes: 2 cycles: 5 _____=>A___P R_ izy
 32 *KIL         $32: CRASH
 33 *RLA izy 8   $33: bytes: 2 cycles: 8 ____P=>A___P RW izy
 34 *NOP zpx 4   $34: bytes: 2 cycles: 4 _____=>_____ R_ zpx
 35 AND zpx 4    $35: bytes: 2 cycles: 4 A____=>A___P R_ zpx
 36 ROL zpx 6    $36: bytes: 2 cycles: 6 ____P=>____P RW zpx
 37 *RLA zpx 6   $37: bytes: 2 cycles: 6 A___P=>A___P RW zpx
 38 SEC 2        $38: bytes: 1 cycles: 2 _____=>____P __
 39 AND aby 4*   $39: bytes: 3 cycles: 4 A____=>A___P R_ absy
 3A *NOP 2       $3A: bytes: 1 cycles: 2 _____=>_____ __
 3B *RLA aby 7   $3B: bytes: 3 cycles: 7 A___P=>A___P RW absy
 3C *NOP abx 4*  $3C: bytes: 3 cycles: 4 _____=>_____ R_ absx
 3D AND abx 4*   $3D: bytes: 3 cycles: 4 A____=>A___P R_ absx
 3E ROL abx 7    $3E: bytes: 3 cycles: 7 ____P=>____P RW absx
 3F *RLA abx 7   $3F: bytes: 3 cycles: 7 A___P=>A___P RW absx
 40 RTI 6        $40: bytes: X cycles: 6 ___S_=>___SP __
 41 EOR izx 6    $41: bytes: 2 cycles: 6 A____=>____P R_ izx
 42 *KIL         $42: CRASH
 43 *SRE izx 8   $43: bytes: 2 cycles: 8 A____=>____P RW izx
 44 *NOP zp 3    $44: bytes: 2 cycles: 3 _____=>_____ R_ zp
 45 EOR zp 3     $45: bytes: 2 cycles: 3 A____=>A___P R_ zp
 46 LSR zp 5     $46: bytes: 2 cycles: 5 _____=>____P RW zp
 47 *SRE zp 5    $47: bytes: 2 cycles: 5 A____=>A___P RW zp
 48 PHA 3        $48: bytes: 1 cycles: 3 A__S_=>___S_ _W
 49 EOR imm 2    $49: bytes: 2 cycles: 2 A____=>A___P __
 4A LSR 2        $4A: bytes: 1 cycles: 2 A____=>A___P __
 4B *ALR imm 2   $4B: bytes: 2 cycles: 2 A____=>A___P __
 4C JMP abs 3    $4C: bytes: X cycles: 3 _____=>_____ __
 4D EOR abs 4    $4D: bytes: 3 cycles: 4 A____=>A___P R_ abs
 4E LSR abs 6    $4E: bytes: 3 cycles: 6 _____=>____P RW abs
 4F *SRE abs 6   $4F: bytes: 3 cycles: 6 A____=>A___P RW abs
 50 BVC rel 2*   $50: bytes: 2 cycles: 3 ____P=>_____ __
 51 EOR izy 5*   $51: bytes: 2 cycles: 5 A____=>____P R_ izy
 52 *KIL         $52: CRASH
 53 *SRE izy 8   $53: bytes: 2 cycles: 8 A____=>____P RW izy
 54 *NOP zpx 4   $54: bytes: 2 cycles: 4 _____=>_____ R_ zpx
 55 EOR zpx 4    $55: bytes: 2 cycles: 4 A____=>A___P R_ zpx
 56 LSR zpx 6    $56: bytes: 2 cycles: 6 _____=>____P RW zpx
 57 *SRE zpx 6   $57: bytes: 2 cycles: 6 A____=>A___P RW zpx
 58 CLI 2        $58: bytes: 1 cycles: 2 _____=>____P __
 59 EOR aby 4*   $59: bytes: 3 cycles: 4 A____=>A___P R_ absy
 5A *NOP 2       $5A: bytes: 1 cycles: 2 _____=>_____ __
 5B *SRE aby 7   $5B: bytes: 3 cycles: 7 A____=>A___P RW absy
 5C *NOP abx 4*  $5C: bytes: 3 cycles: 4 _____=>_____ R_ absx
 5D EOR abx 4*   $5D: bytes: 3 cycles: 4 A____=>A___P R_ absx
 5E LSR abx 7    $5E: bytes: 3 cycles: 7 _____=>____P RW absx
 5F *SRE abx 7   $5F: bytes: 3 cycles: 7 A____=>A___P RW absx
 60 RTS 6        $60: bytes: X cycles: 6 ___S_=>___S_ __
 62 *KIL         $62: CRASH
 63 *RRA izx 8   $63: bytes: 2 cycles: 8 A___P=>A___P RW izx
 64 *NOP zp 3    $64: bytes: 2 cycles: 3 _____=>_____ R_ zp
 66 ROR zp 5     $66: bytes: 2 cycles: 5 ____P=>____P RW zp
 67 *RRA zp 5    $67: bytes: 2 cycles: 5 A___P=>A___P RW zp
 68 PLA 4        $68: bytes: 1 cycles: 4 ___S_=>A__SP __
 6A ROR 2        $6A: bytes: 1 cycles: 2 A___P=>A___P __
 6B *ARR imm 2   $6B: bytes: 2 cycles: 2 A___P=>A___P __
 6C JMP ind 5    $6C: bytes: X cycles: 5 _____=>_____ __
 6E ROR abs 6    $6E: bytes: 3 cycles: 6 ____P=>____P RW abs
 6F *RRA abs 6   $6F: bytes: 3 cycles: 6 A___P=>A___P RW abs
 70 BVS rel 2*   $70: bytes: 2 cycles: 2 _____=>_____ __
 72 *KIL         $72: CRASH
 73 *RRA izy 8   $73: bytes: 2 cycles: 8 A___P=>A___P RW izy
 74 *NOP zpx 4   $74: bytes: 2 cycles: 4 _____=>_____ R_ zpx
 76 ROR zpx 6    $76: bytes: 2 cycles: 6 ____P=>____P RW zpx
 77 *RRA zpx 6   $77: bytes: 2 cycles: 6 A___P=>A___P RW zpx
 78 SEI 2        $78: bytes: 1 cycles: 2 _____=>____P __
 7A *NOP 2       $7A: bytes: 1 cycles: 2 _____=>_____ __
 7B *RRA aby 7   $7B: bytes: 3 cycles: 7 A___P=>A___P RW absy
 7C *NOP abx 4*  $7C: bytes: 3 cycles: 4 _____=>_____ R_ absx
 7E ROR abx 7    $7E: bytes: 3 cycles: 7 ____P=>____P RW absx
 7F *RRA abx 7   $7F: bytes: 3 cycles: 7 A___P=>A___P RW absx
 80 *NOP imm 2   $80: bytes: 2 cycles: 2 _____=>_____ __
 82 *NOP imm 2   $82: bytes: 2 cycles: 2 _____=>_____ __
 83 *SAX izx 6   $83: bytes: 2 cycles: 6 _____=>_____ RW izx
 87 *SAX zp 3    $87: bytes: 2 cycles: 3 _____=>_____ _W zp
 89 *NOP imm 2   $89: bytes: 2 cycles: 2 _____=>_____ __
 8A TXA 2        $8A: bytes: 1 cycles: 2 _X___=>A___P __
 8B *XAA imm 2   $8B: bytes: 2 cycles: 2 _____=>A___P __
 8F *SAX abs 4   $8F: bytes: 3 cycles: 4 _____=>_____ _W abs
 90 BCC rel 2*   $90: bytes: 2 cycles: 3 ____P=>_____ __
 92 *KIL         $92: CRASH
 93 *AHX izy 6   $93: bytes: 2 cycles: 6 _____=>_____ RW izy
 97 *SAX zpy 4   $97: bytes: 2 cycles: 4 _____=>_____ RW zpy
 98 TYA 2        $98: bytes: 1 cycles: 2 __Y__=>A___P __
 9A TXS 2        $9A: bytes: X cycles: 2 _X___=>___S_ __
 9B *TAS aby 5   $9B: bytes: X cycles: 5 __Y__=>___S_ _W
 9C *SHY abx 5   $9C: bytes: 3 cycles: 5 __Y__=>_____ RW absx
 9E *SHX aby 5   $9E: bytes: 3 cycles: 5 _X___=>_____ RW absy
 9F *AHX aby 5   $9F: bytes: 3 cycles: 5 _____=>_____ RW absy
 A3 *LAX izx 6   $A3: bytes: 2 cycles: 6 _____=>AX__P R_ izx
 A7 *LAX zp 3    $A7: bytes: 2 cycles: 3 _____=>AX__P R_ zp
 A8 TAY 2        $A8: bytes: 1 cycles: 2 A____=>__Y_P __
 AA TAX 2        $AA: bytes: 1 cycles: 2 A____=>_X__P __
 AB *LAX imm 2   $AB: bytes: 2 cycles: 2 A____=>AX__P __
 AF *LAX abs 4   $AF: bytes: 3 cycles: 4 _____=>AX__P R_ abs
 B0 BCS rel 2*   $B0: bytes: 2 cycles: 2 _____=>_____ __
 B2 *KIL         $B2: CRASH
 B3 *LAX izy 5*  $B3: bytes: 2 cycles: 5 _____=>AX__P R_ izy
 B7 *LAX zpy 4   $B7: bytes: 2 cycles: 4 _____=>AX__P R_ zpy
 B8 CLV 2        $B8: bytes: 1 cycles: 2 _____=>____P __
 BA TSX 2        $BA: bytes: 1 cycles: 2 ___S_=>_X__P __
 BB *LAS aby 4*  $BB: bytes: 3 cycles: 4 ___S_=>AX_SP R_ absy
 BF *LAX aby 4*  $BF: bytes: 3 cycles: 4 _____=>AX__P R_ absy
 C0 CPY imm 2    $C0: bytes: 2 cycles: 2 __Y__=>____P __
 C1 CMP izx 6    $C1: bytes: 2 cycles: 6 A____=>____P R_ izx
 C2 *NOP imm 2   $C2: bytes: 2 cycles: 2 _____=>_____ __
 C3 *DCP izx 8   $C3: bytes: 2 cycles: 8 A____=>____P RW izx
 C4 CPY zp 3     $C4: bytes: 2 cycles: 3 __Y__=>____P R_ zp
 C5 CMP zp 3     $C5: bytes: 2 cycles: 3 A____=>____P R_ zp
 C6 DEC zp 5     $C6: bytes: 2 cycles: 5 _____=>____P RW zp
 C7 *DCP zp 5    $C7: bytes: 2 cycles: 5 A____=>____P RW zp
 C8 INY 2        $C8: bytes: 1 cycles: 2 __Y__=>__Y_P __
 C9 CMP imm 2    $C9: bytes: 2 cycles: 2 A____=>____P __
 CB *AXS imm 2   $CB: bytes: 2 cycles: 2 _____=>_X__P __
 CC CPY abs 4    $CC: bytes: 3 cycles: 4 __Y__=>____P R_ abs
 CD CMP abs 4    $CD: bytes: 3 cycles: 4 A____=>____P R_ abs
 CE DEC abs 6    $CE: bytes: 3 cycles: 6 _____=>____P RW abs
 CF *DCP abs 6   $CF: bytes: 3 cycles: 6 A____=>____P RW abs
 D1 CMP izy 5*   $D1: bytes: 2 cycles: 5 A____=>____P R_ izy
 D2 *KIL         $D2: CRASH
 D3 *DCP izy 8   $D3: bytes: 2 cycles: 8 A____=>____P RW izy
 D4 *NOP zpx 4   $D4: bytes: 2 cycles: 4 _____=>_____ R_ zpx
 D5 CMP zpx 4    $D5: bytes: 2 cycles: 4 A____=>____P R_ zpx
 D6 DEC zpx 6    $D6: bytes: 2 cycles: 6 _____=>____P RW zpx
 D7 *DCP zpx 6   $D7: bytes: 2 cycles: 6 A____=>____P RW zpx
 D8 CLD 2        $D8: bytes: 1 cycles: 2 _____=>____P __
 D9 CMP aby 4*   $D9: bytes: 3 cycles: 4 A____=>____P R_ absy
 DA *NOP 2       $DA: bytes: 1 cycles: 2 _____=>_____ __
 DB *DCP aby 7   $DB: bytes: 3 cycles: 7 A____=>____P RW absy
 DC *NOP abx 4*  $DC: bytes: 3 cycles: 4 _____=>_____ R_ absx
 DD CMP abx 4*   $DD: bytes: 3 cycles: 4 A____=>____P R_ absx
 DE DEC abx 7    $DE: bytes: 3 cycles: 7 _____=>____P RW absx
 DF *DCP abx 7   $DF: bytes: 3 cycles: 7 A____=>____P RW absx
 E0 CPX imm 2    $E0: bytes: 2 cycles: 2 _X___=>____P __
 E1 SBC izx 6    $E1: bytes: 2 cycles: 6 A___P=>A___P R_ izx
 E2 *NOP imm 2   $E2: bytes: 2 cycles: 2 _____=>_____ __
 E3 *ISC izx 8   $E3: bytes: 2 cycles: 8 A___P=>A___P RW izx
 E4 CPX zp 3     $E4: bytes: 2 cycles: 3 _X___=>____P R_ zp
 E5 SBC zp 3     $E5: bytes: 2 cycles: 3 A___P=>A___P R_ zp
 E6 INC zp 5     $E6: bytes: 2 cycles: 5 _____=>____P RW zp
 E7 *ISC zp 5    $E7: bytes: 2 cycles: 5 A___P=>A___P RW zp
 E8 INX 2        $E8: bytes: 1 cycles: 2 _X___=>_X__P __
 E9 SBC imm 2    $E9: bytes: 2 cycles: 2 A___P=>A___P __
 EA NOP 2        $EA: bytes: 1 cycles: 2 _____=>_____ __
 EB *SBC imm 2   $EB: bytes: 2 cycles: 2 A___P=>A___P __
 EC CPX abs 4    $EC: bytes: 3 cycles: 4 _X___=>____P R_ abs
 ED SBC abs 4    $ED: bytes: 3 cycles: 4 A___P=>A___P R_ abs
 EE INC abs 6    $EE: bytes: 3 cycles: 6 _____=>____P RW abs
 EF *ISC abs 6   $EF: bytes: 3 cycles: 6 A___P=>A___P RW abs
 F0 BEQ rel 2*   $F0: bytes: 2 cycles: 2 _____=>_____ __
 F1 SBC izy 5*   $F1: bytes: 2 cycles: 5 A___P=>A___P R_ izy
 F2 *KIL         $F2: CRASH
 F3 *ISC izy 8   $F3: bytes: 2 cycles: 8 A___P=>A___P RW izy
 F4 *NOP zpx 4   $F4: bytes: 2 cycles: 4 _____=>_____ R_ zpx
 F5 SBC zpx 4    $F5: bytes: 2 cycles: 4 A___P=>A___P R_ zpx
 F6 INC zpx 6    $F6: bytes: 2 cycles: 6 _____=>____P RW zpx
 F7 *ISC zpx 6   $F7: bytes: 2 cycles: 6 A___P=>A___P RW zpx
 F8 SED 2        $F8: bytes: 1 cycles: 2 _____=>____P __
 F9 SBC aby 4*   $F9: bytes: 3 cycles: 4 A___P=>A___P R_ absy
 FA *NOP 2       $FA: bytes: 1 cycles: 2 _____=>_____ __
 FB *ISC aby 7   $FB: bytes: 3 cycles: 7 A___P=>A___P RW absy
 FC *NOP abx 4*  $FC: bytes: 3 cycles: 4 _____=>_____ R_ absx
 FD SBC abx 4*   $FD: bytes: 3 cycles: 4 A___P=>A___P R_ absx
 FE INC abx 7    $FE: bytes: 3 cycles: 7 _____=>____P RW absx
 FF *ISC abx     $FF: bytes: 3 cycles: 7 A___P=>A___P RW absx
*/
