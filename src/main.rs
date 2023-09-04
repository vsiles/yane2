use crate::miniquad::log;
use macroquad::prelude::*;
use std::collections::BTreeMap;

mod bus;
mod cpu;

use bus::Bus;
use cpu::Cpu;

const MAC_BORDER: f32 = 28.0;
const FONT_SIZE: u16 = 16;
const H_STEP: f32 = 1.0 + FONT_SIZE as f32;

#[macroquad::main("Yane")]
async fn main() {
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

    let bus = Bus::new();

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
    let mut disas = BTreeMap::new();
    for i in 0x6000..0x9000 {
        disas.insert(i, format!("${:>04X}: BRK #$00 {{IMM}}", i));
    }

    loop {
        if is_key_down(KeyCode::Q) || is_key_down(KeyCode::Escape) {
            break;
        }

        clear_background(BLUE);

        // texture.update(&image);
        // draw_texture(&texture, 0., 0., WHITE);

        draw_ram(10.0, MAC_BORDER + 10.0, 0x0000, &bus, 16, 16, &font_params).await;
        draw_ram(
            10.0,
            20.0 * H_STEP + 10.0,
            0x8000,
            &bus,
            16,
            16,
            &font_params,
        )
        .await;
        draw_cpu(600.0, MAC_BORDER + 10.0, &bus.cpu, &font_params).await;
        draw_code(
            600.0,
            MAC_BORDER + 10.0 + 7.0 * H_STEP,
            0x7000,
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
        if cpu.get_flag(cpu::Flags::N) {
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
        if cpu.get_flag(cpu::Flags::V) {
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
        if cpu.get_flag(cpu::Flags::U) {
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
        if cpu.get_flag(cpu::Flags::B) {
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
        if cpu.get_flag(cpu::Flags::D) {
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
        if cpu.get_flag(cpu::Flags::I) {
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
        if cpu.get_flag(cpu::Flags::Z) {
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
        if cpu.get_flag(cpu::Flags::C) {
            green.clone()
        } else {
            red.clone()
        },
    );

    pos = y + H_STEP;

    draw_text_ex(
        &format!("PC: ${:>04X}", cpu.pc),
        x,
        pos,
        font_params.clone(),
    );
    pos += H_STEP;
    draw_text_ex(
        &format!("A: ${:>02X} [{}]", cpu.a, cpu.a),
        x,
        pos,
        font_params.clone(),
    );
    pos += H_STEP;
    draw_text_ex(
        &format!("X: ${:>02X} [{}]", cpu.x, cpu.x),
        x,
        pos,
        font_params.clone(),
    );
    pos += H_STEP;
    draw_text_ex(
        &format!("Y: ${:>02X} [{}]", cpu.y, cpu.y),
        x,
        pos,
        font_params.clone(),
    );
    pos += H_STEP;
    draw_text_ex(
        &format!("Stack P: ${:>04X}", cpu.sp),
        x,
        pos,
        font_params.clone(),
    );
}

async fn draw_ram(
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
