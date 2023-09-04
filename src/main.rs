use crate::miniquad::log;
use macroquad::prelude::*;

mod bus;
mod cpu;

use bus::Bus;
use cpu::Cpu;

const MAC_BORDER : f32 = 28.0;
const FONT_SIZE: u16 = 16;

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

    let font = load_ttf_font("./resources/fonts/DejaVuSansMono.ttf").await.unwrap();

    let font_params = TextParams {
        font_size: FONT_SIZE,
        font: Some(&font),
        ..Default::default()
    };

    loop {
        if is_key_down(KeyCode::Q) || is_key_down(KeyCode::Escape) {
            break
        }

        clear_background(BLUE);

        // texture.update(&image);
        // draw_texture(&texture, 0., 0., WHITE);
        draw_cpu(800.0, 50.0, &bus.cpu, &font_params).await;

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
    draw_text_ex("N", pos, y, if cpu.get_flag(cpu::Flags::N) { green.clone() } else { red.clone() });
    pos += xstep;
    draw_text_ex("V", pos, y, if cpu.get_flag(cpu::Flags::V) { green.clone() } else { red.clone() });
    pos += xstep;
    draw_text_ex("-", pos, y, if cpu.get_flag(cpu::Flags::U) { green.clone() } else { red.clone() });
    pos += xstep;
    draw_text_ex("B", pos, y, if cpu.get_flag(cpu::Flags::B) { green.clone() } else { red.clone() });
    pos += xstep;
    draw_text_ex("D", pos, y, if cpu.get_flag(cpu::Flags::D) { green.clone() } else { red.clone() });
    pos += xstep;
    draw_text_ex("I", pos, y, if cpu.get_flag(cpu::Flags::I) { green.clone() } else { red.clone() });
    pos += xstep;
    draw_text_ex("Z", pos, y, if cpu.get_flag(cpu::Flags::Z) { green.clone() } else { red.clone() });
    pos += xstep;
    draw_text_ex("C", pos, y, if cpu.get_flag(cpu::Flags::C) { green.clone() } else { red.clone() });

    let ystep : f32 = 1.0 + FONT_SIZE as f32;
    pos = y + ystep;

    draw_text_ex(&format!("PC: ${:#>04X}", cpu.pc), x, pos, font_params.clone());
    pos += ystep;
    draw_text_ex(&format!("A: ${:#>02X} [{}]", cpu.a, cpu.a), x, pos, font_params.clone());
    pos += ystep;
    draw_text_ex(&format!("X: ${:#>02X} [{}]", cpu.x, cpu.x), x, pos, font_params.clone());
    pos += ystep;
    draw_text_ex(&format!("Y: ${:#>02X} [{}]", cpu.y, cpu.y), x, pos, font_params.clone());
    pos += ystep;
    draw_text_ex(&format!("Stack P: ${:#>04X}", cpu.sp), x, pos, font_params.clone());
}

/*
asyn fn draw_ram(x: f32, y: f32, ram_addr: u16, rows: usize, columns: usize) {
        int nRamX = x, nRamY = y;
        for (int row = 0; row < nRows; row++)
        {
            std::string sOffset = "$" + hex(nAddr, 4) + ":";
            for (int col = 0; col < nColumns; col++)
            {
                sOffset += " " + hex(nes.read(nAddr, true), 2);
                nAddr += 1;
            }
            DrawString(nRamX, nRamY, sOffset);
            nRamY += 10;
        }
    }

    void DrawCode(int x, int y, int nLines)
    {
        auto it_a = mapAsm.find(nes.cpu.pc);
        int nLineY = (nLines >> 1) * 10 + y;
        if (it_a != mapAsm.end())
        {
            DrawString(x, nLineY, (*it_a).second, olc::CYAN);
            while (nLineY < (nLines * 10) + y)
            {
                nLineY += 10;
                if (++it_a != mapAsm.end())
                {
                    DrawString(x, nLineY, (*it_a).second);
                }
            }
        }

        it_a = mapAsm.find(nes.cpu.pc);
        nLineY = (nLines >> 1) * 10 + y;
        if (it_a != mapAsm.end())
        {
            while (nLineY > y)
            {
                nLineY -= 10;
                if (--it_a != mapAsm.end())
                {
                    DrawString(x, nLineY, (*it_a).second);
                }
            }
        }
    }
*/
