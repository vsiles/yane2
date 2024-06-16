#![allow(dead_code)]
//! <https://www.nesdev.org/wiki/INES>
//! This file is just an encoding of the iNes file format

use eyre::{Result, ensure};
use std::{fs, path::Path};

/// The format of the header is as follows:
/// Bytes Description
/// 0-3 Constant $4E $45 $53 $1A (ASCII "NES" followed by MS-DOS end-of-file)
/// 4 Size of PRG ROM in 16 KB units
/// 5 Size of CHR ROM in 8 KB units (value 0 means the board uses CHR RAM)
/// 6 Flags 6 – Mapper, mirroring, battery, trainer
/// 7 Flags 7 – Mapper, VS/Playchoice, NES 2.0
/// 8 Flags 8 – PRG-RAM size (rarely used extension)
/// 9 Flags 9 – TV system (rarely used extension)
/// 10 Flags 10 – TV system, PRG-RAM presence (unofficial, rarely used extension)
/// 11-15 Unused padding (should be filled with zero, but some rippers put their name across bytes 7-15) 
#[derive(Debug)]
pub struct Header {
    prg_rom_size: usize, // bytes
    chr_rom_size: usize, // bytes
    // From Flag 6 (low) and Flag 7 (up)
    mapper_number: u8,
    // Flag 6
    nametable_arrangement: NametableArrangement,
    battery_backed_prg_ram: bool,
    /// 512-byte trainer at $7000-$71FF (stored before PRG data)
    trainer_512: bool,
    alternative_nametable_layout: bool,
    // Flag 7
    vs_unisystem: bool,
    playchoice_10: bool,
    nes_2_0: bool,
    // Flag 8
    prg_ram_size: usize,
    // Flag 9
    tv_system: TvSystem,
    // Flag 10
    // This byte is not part of the official specification, and relatively few emulators honor it. 
    tv_system_ext: TvSystemExt,
    high_prg_ram: bool, // PRG RAM ($6000-$7FFF) (0: present; 1: not present)
    bus_conflicts: bool,
}

fn is_bit_set(value: u8, bit: u8) -> bool {
    (value & (1 << bit)) != 0
}
impl Header {
    pub fn new(header: [u8; 16]) -> Result<Self> {
        ensure!(header[0] == 0x4e, "Invalid magic word 0");
        ensure!(header[1] == 0x45, "Invalid magic word 1");
        ensure!(header[2] == 0x53, "Invalid magic word 2");
        ensure!(header[3] == 0x1a, "Invalid magic word 3");

        // 4 Size of PRG ROM in 16 KB units
        let prg_rom_size = (header[4] as usize) * 16 * 1024;
        // 5 Size of CHR ROM in 8 KB units (value 0 means the board uses CHR RAM)
        let chr_rom_size = (header[5] as usize) * 8 * 1024;

        let flag6 = header[6];
        let low_mapper_number = flag6 >> 4;
        let nametable_arrangement = if is_bit_set(flag6, 0) {
            NametableArrangement::Horizontal
        } else {
                NametableArrangement::Vertical
        };
        let battery_backed_prg_ram = is_bit_set(flag6, 1);
        let trainer_512 = is_bit_set(flag6, 2);
        let alternative_nametable_layout = is_bit_set(flag6, 3);

        let flag7 = header[7];
        let high_mapper_number = flag7 >> 4;
        let vs_unisystem = is_bit_set(flag7, 0);
        let playchoice_10 = is_bit_set(flag7, 1);
        let nes_2_0 = (flag7 >> 2) & 0b11 == 2;
        ensure!(!playchoice_10, "Playchoice detected, not supported");
        ensure!(!nes_2_0, "Nes 2.0 format detected. Not supported");

        let mapper_number = low_mapper_number | (high_mapper_number << 4);
        // TODO: double check
        let prg_ram_size = (if header[8] == 0 { 1 } else { header[8] as usize }) * 8 * 1024;

        let flag9 = header[9];
        let tv_system = if is_bit_set(flag9, 0) {
            TvSystem::Ntsc
        } else {
            TvSystem::Pal
        };
        ensure!((flag9 >> 1) & 0b01111111 == 0, "invalid flag9, expecting zeroes");

        let flag10 = header[10];
        let tv_system_ext = match flag10 & 0b11 {
            0 => TvSystemExt::Legacy(TvSystem::Ntsc),
            2 => TvSystemExt::Legacy(TvSystem::Pal),
            _ => TvSystemExt::Dual,
        };
        let high_prg_ram = is_bit_set(flag10, 4);
        let bus_conflicts = is_bit_set(flag10, 5);

        Ok(Self {
            prg_rom_size,
            chr_rom_size,
            mapper_number,
            nametable_arrangement,
            battery_backed_prg_ram,
            trainer_512,
            alternative_nametable_layout,
            vs_unisystem,
            playchoice_10,
            nes_2_0,
            prg_ram_size,
            tv_system,
            tv_system_ext,
            high_prg_ram,
            bus_conflicts,
        })
    }
}

/// An iNES file consists of the following sections, in order:
/// 
///     Header (16 bytes)
///     Trainer, if present (0 or 512 bytes)
///     PRG ROM data (16384 * x bytes)
///     CHR ROM data, if present (8192 * y bytes)
///     PlayChoice INST-ROM, if present (0 or 8192 bytes)
///     PlayChoice PROM, if present (16 bytes Data, 16 bytes CounterOut) (this is often missing; see PC10 ROM-Images for details)
/// 
/// Some ROM-Images additionally contain a 128-byte (or sometimes 127-byte) title at the end of the file.
#[derive(Debug)]
pub struct INes {
    pub header: Header,
    trainer: Option<[u8; 512]>,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    // TODO: playchoice rom data
}

impl INes {
    pub fn new(file: &Path) -> Result<INes> {
        let mut bytes = fs::read(file)?;
        ensure!(bytes.len() > 16, "Missing iNes header");
        let mut header_bytes = [0; 16];
        header_bytes.copy_from_slice(&bytes[0..16]);
        bytes = bytes.split_off(16);

        let header = Header::new(header_bytes)?;

        let trainer = header.trainer_512.then_some({
            let mut trainer_bytes = [0; 512];
            trainer_bytes.copy_from_slice(&bytes[0..512]);
            bytes = bytes.split_off(512);
            trainer_bytes
        });

        let rest = bytes.split_off(header.prg_rom_size);
        let prg_rom = bytes;
        bytes = rest;

        let _rest = bytes.split_off(header.chr_rom_size);
        let chr_rom = bytes;

        Ok(Self {
            header,
            trainer,
            prg_rom,
            chr_rom,
        })
    }
}

#[derive(Debug)]
pub enum NametableArrangement {
    Vertical, // PPU A11
    Horizontal, // PPU A10
}

#[derive(Debug)]
pub enum TvSystem {
    Ntsc,
    Pal,
}

// Not Official
#[derive(Debug)]
pub enum TvSystemExt {
    Legacy(TvSystem),
    Dual
}
