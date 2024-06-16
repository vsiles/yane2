use crate::miniquad::log;
use macroquad::prelude::*;
use std::collections::BTreeMap;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use eyre::Result;

mod bus;
mod cpu;
mod ines;

use bus::Bus;
use cpu::Cpu;
use ines::INes;

const MAC_BORDER: f32 = 28.0;
const FONT_SIZE: u16 = 16;
const H_STEP: f32 = 1.0 + FONT_SIZE as f32;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    test: TestCommand,
}

#[derive(Subcommand)]
enum TestCommand {
    Test0,
    Nestest {
        #[arg(short, long)]
        path: PathBuf,
    },
}

#[macroquad::main("Yane")]
async fn main() -> Result<()> {
    request_new_screen_size(1024.0, 768.0 + MAC_BORDER);

    next_frame().await; // acknowledge new screen size

    // let w = (screen_width() as usize) / 2;
    // let h = (screen_height() as usize) / 2;

    log!(
        log::Level::Info,
        "screen size is {} x {}",
        screen_width(),
        screen_height()
    );

    let cli = Cli::parse();

    let mut cpu = match cli.test {
        TestCommand::Test0 => test0(),
        TestCommand::Nestest { path } => nestest(&path)?,
    };

    let disas = cpu.disassemble(0x0000, 0xFFFF);

    // let image = Image::gen_image_color(w as u16, h as u16, RED);
    // let texture = Texture2D::from_image(&image);

    let font = load_ttf_font("./resources/fonts/DejaVuSansMono.ttf")
        .await
        .unwrap();

    let font_params = TextParams {
        font_size: FONT_SIZE,
        font: Some(&font),
        ..Default::default()
    };

    // just for test

    loop {
        if is_key_down(KeyCode::Q) || is_key_down(KeyCode::Escape) {
            break;
        }

        if is_key_pressed(KeyCode::Space) {
            loop {
                cpu.clock();
                if cpu.complete() {
                    break;
                }
            }
        }

        if is_key_pressed(KeyCode::R) {
            cpu.reset()
        }

        // TODO: IRQ / NMI

        clear_background(BLUE);

        // texture.update(&image);
        // draw_texture(&texture, 0., 0., WHITE);

        draw_ram(
            10.0,
            MAC_BORDER + 10.0,
            0x0000,
            &cpu.bus().read().expect("Failed to get bus"),
            16,
            16,
            &font_params,
        );

        draw_ram(
            10.0,
            20.0 * H_STEP + 10.0,
            0x8000,
            &cpu.bus().read().expect("Failed to get bus"),
            16,
            16,
            &font_params,
        );

        draw_cpu(600.0, MAC_BORDER + 10.0, &cpu, &font_params).await;
        draw_code(
            600.0,
            MAC_BORDER + 10.0 + 7.0 * H_STEP,
            cpu.core.pc,
            26,
            &disas,
            &font_params,
        )
        .await;

        draw_text_ex(
            "SPACE = Step Instruction    R = RESET    I = IRQ    N = NMI",
            40.0,
            700.0,
            font_params.clone(),
        );

        next_frame().await
    }

    Ok(())
}

fn test0() -> Cpu {
    let mut bus = Bus::new();

    // TODO: implement proper ROM loading
    // example program is from https://github.com/OneLoneCoder/olcNES

    let program =
        "A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA"
            .split(' ');
    let mut addr = 0x8000;
    for s in program {
        let byte = u8::from_str_radix(s, 16).unwrap();
        bus.ram[addr] = byte;
        addr += 1;
    }

    // Set Reset Vector
    bus.ram[0xFFFC] = 0x00;
    bus.ram[0xFFFD] = 0x80;

    let mut cpu = Cpu::new(bus);
    cpu.reset();
    cpu
}

fn nestest(path: &Path) -> Result<Cpu> {
    let mut bus = Bus::new();

    let nestest = INes::new(path)?;

    eprintln!("{:?}", nestest.header);

    // Set Reset Vector
    bus.ram[0xFFFC] = 0x00;
    bus.ram[0xFFFD] = 0x80;

    let mut cpu = Cpu::new(bus);
    cpu.reset();
    cpu.core.pc = 0xC000;
    Ok(cpu)

}

async fn draw_cpu(x: f32, y: f32, cpu: &Cpu, font_params: &TextParams<'_>) {
    let red = TextParams {
        color: RED,
        ..font_params.clone()
    };
    let green = TextParams {
        color: GREEN,
        ..font_params.clone()
    };

    let mut pos = x;
    let xstep = 12.0;
    draw_text_ex("STATUS: ", pos, y, font_params.clone());
    pos += 75.0;
    draw_text_ex(
        "N",
        pos,
        y,
        if cpu.core.get_flag(cpu::Flags::N) {
            green.clone()
        } else {
            red.clone()
        },
    );
    pos += xstep;
    draw_text_ex(
        "V",
        pos,
        y,
        if cpu.core.get_flag(cpu::Flags::V) {
            green.clone()
        } else {
            red.clone()
        },
    );
    pos += xstep;
    draw_text_ex(
        "-",
        pos,
        y,
        if cpu.core.get_flag(cpu::Flags::U) {
            green.clone()
        } else {
            red.clone()
        },
    );
    pos += xstep;
    draw_text_ex(
        "B",
        pos,
        y,
        if cpu.core.get_flag(cpu::Flags::B) {
            green.clone()
        } else {
            red.clone()
        },
    );
    pos += xstep;
    draw_text_ex(
        "D",
        pos,
        y,
        if cpu.core.get_flag(cpu::Flags::D) {
            green.clone()
        } else {
            red.clone()
        },
    );
    pos += xstep;
    draw_text_ex(
        "I",
        pos,
        y,
        if cpu.core.get_flag(cpu::Flags::I) {
            green.clone()
        } else {
            red.clone()
        },
    );
    pos += xstep;
    draw_text_ex(
        "Z",
        pos,
        y,
        if cpu.core.get_flag(cpu::Flags::Z) {
            green.clone()
        } else {
            red.clone()
        },
    );
    pos += xstep;
    draw_text_ex(
        "C",
        pos,
        y,
        if cpu.core.get_flag(cpu::Flags::C) {
            green.clone()
        } else {
            red.clone()
        },
    );

    pos = y + H_STEP;

    draw_text_ex(
        &format!("PC: ${:>04X}", cpu.core.pc),
        x,
        pos,
        font_params.clone(),
    );
    pos += H_STEP;
    draw_text_ex(
        &format!("A: ${:>02X} [{}]", cpu.core.a, cpu.core.a),
        x,
        pos,
        font_params.clone(),
    );
    pos += H_STEP;
    draw_text_ex(
        &format!("X: ${:>02X} [{}]", cpu.core.x, cpu.core.x),
        x,
        pos,
        font_params.clone(),
    );
    pos += H_STEP;
    draw_text_ex(
        &format!("Y: ${:>02X} [{}]", cpu.core.y, cpu.core.y),
        x,
        pos,
        font_params.clone(),
    );
    pos += H_STEP;
    draw_text_ex(
        &format!("Stack P: ${:>04X}", cpu.core.sp),
        x,
        pos,
        font_params.clone(),
    );
}

fn draw_ram(
    x: f32,
    y: f32,
    ram_addr: u16,
    bus: &Bus,
    rows: usize,
    columns: usize,
    font_params: &TextParams<'_>,
) {
    let mut pos = y;
    let mut addr = ram_addr;
    for _ in 0..rows {
        let mut line = format!("${:>04X}:", addr);
        for _ in 0..columns {
            line = format!("{} {:>02X}", line, bus.read(addr));
            addr += 1;
        }
        draw_text_ex(&line, x, pos, font_params.clone());
        pos += H_STEP;
    }
}

async fn draw_code(
    x: f32,
    y: f32,
    pc: u16,
    num_lines: u16,
    disas: &BTreeMap<u16, String>,
    font_params: &TextParams<'_>,
) {
    let cyan = TextParams {
        color: GREEN,
        ..font_params.clone()
    };

    let mut iter = disas.iter().skip_while(|(addr, _)| **addr < pc);

    let nr = num_lines / 2;

    let mut pos = y + (nr as f32) * H_STEP;

    // pc
    match iter.next() {
        None => (),
        Some((_, line)) => {
            draw_text_ex(line, x, pos, cyan);
            pos += H_STEP
        }
    }

    // instructions after pc
    for _ in 0..nr {
        let (_addr, line) = match iter.next() {
            None => break,
            Some(x) => x,
        };
        draw_text_ex(line, x, pos, font_params.clone());
        pos += H_STEP;
    }

    // instructions before pc
    let mut iter = disas.iter().rev().skip_while(|(addr, _)| **addr > pc);

    // skip pc
    let _ = iter.next();

    let mut pos = y + ((nr - 1) as f32) * H_STEP;
    for _ in 0..nr {
        let (_addr, line) = match iter.next() {
            None => break,
            Some(x) => x,
        };
        draw_text_ex(line, x, pos, font_params.clone());
        pos -= H_STEP;
    }

    // auto it_a = mapAsm.find(nes.cpu.pc);
    // int nLineY = (nLines >> 1) * 10 + y;
    // if (it_a != mapAsm.end())
    // {
    //     DrawString(x, nLineY, (*it_a).second, olc::CYAN);
    //     while (nLineY < (nLines * 10) + y)
    //     {
    //         nLineY += 10;
    //         if (++it_a != mapAsm.end())
    //         {
    //             DrawString(x, nLineY, (*it_a).second);
    //         }
    //     }
    // }

    // it_a = mapAsm.find(nes.cpu.pc);
    // nLineY = (nLines >> 1) * 10 + y;
    // if (it_a != mapAsm.end())
    // {
    //     while (nLineY > y)
    //     {
    //         nLineY -= 10;
    //         if (--it_a != mapAsm.end())
    //         {
    //             DrawString(x, nLineY, (*it_a).second);
    //         }
    //     }
    // }
}
