extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

use std::time::Duration;

pub fn main() -> Result<(), String> {
  let sdl_context = sdl2::init()?;
  let video_subsystem = sdl_context.video()?;

  let window = video_subsystem
    .window("rust-sdl2 demo: Video", 512, 512)
    .position_centered()
    .opengl()
    .build()
    .map_err(|e| e.to_string())?;

  let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
  let texture_creator = canvas.texture_creator();

  let mut texture = texture_creator
    .create_texture_streaming(PixelFormatEnum::RGB24, 512, 512)
    .map_err(|e| e.to_string())?;

  // Raycaster code port
  let win_w = 512;
  let win_h = 512;

  let map_w = 16;
  let map_h = 16;

  let mut framebuffer: [u8; 512 * 512 * 3] = [0x00; 512 * 512 * 3];

  let map = "0000222222220000\
             1              0\
             1      11111   0\
             1     0        0\
             0     0  1110000\
             0     3        0\
             0   10000      0\
             0   0   11100  0\
             0   0   0      0\
             0   0   1  00000\
             0       1      0\
             2       1      0\
             0       0      0\
             0 0000000      0\
             0              0\
             0002222222200000";

  let player_x = 3.456_f32;
  let player_y = 2.345_f32;
  let player_a = 1.523_f32;

  let mut event_pump = sdl_context.event_pump()?;

  'running: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. }
        | Event::KeyDown {
          keycode: Some(Keycode::Escape),
          ..
        } => break 'running,
        _ => {}
      }
    }

    canvas.clear();

    // Game loop START

    // Draw bg
    for j in 0..win_w {
      for i in 0..win_h {
        let r = 255.0 * j as f32 / win_h as f32;
        let g = 255.0 * i as f32 / win_w as f32;
        let b = 0;

        let offset = j * (3 * 512) + i * 3;

        framebuffer[offset] = r as u8;
        framebuffer[offset + 1] = g as u8;
        framebuffer[offset + 2] = b as u8;
      }
    }

    let rect_w = win_w / map_w;
    let rect_h = win_h / map_h;

    // Draw map
    for j in 0..map_h {
      for i in 0..map_w {
        if map.chars().nth(i + j * map_w).unwrap() == ' ' {
          continue;
        }

        let rect_x = i * rect_w;
        let rect_y = j * rect_h;

        draw_rectangle(
          &mut framebuffer,
          win_w,
          win_h,
          rect_x,
          rect_y,
          rect_w,
          rect_h,
          sdl2::pixels::Color::RGB(0, 255, 255),
        );
      }
    }

    // Draw player
    draw_rectangle(
      &mut framebuffer,
      win_w,
      win_h,
      (player_x * rect_w as f32) as usize,
      (player_y * rect_h as f32) as usize,
      5,
      5,
      sdl2::pixels::Color::RGB(255, 255, 255),
    );

    // Draw ray from player to wall

    let mut t = 0.0_f32;
    loop {
      let cx = player_x + t * player_a.cos();
      let cy = player_y + t * player_a.sin();

      let map_index = cx as usize + cy as usize * map_w;

      println!("map index: {}", map_index);

      let map_char = map.chars().nth(map_index).unwrap();

      println!("map char: {}", map_char);

      if map_char != ' ' {
        break;
      }

      let pix_x = cx as usize * rect_w;
      let pix_y = cy as usize * rect_h;

      let offset = (pix_x + pix_y*win_w) * 3;

      framebuffer[offset] = 255;
      framebuffer[offset + 1] = 255;
      framebuffer[offset + 2] = 255;

      t = t + 0.05;

      println!("t: {}", t);

      if t > 20.0 {
        println!("breaking? {}", t);
        // break;
      }
    }

    panic!("hi");


    // Game loop END

    texture
      .update(Rect::new(0, 0, 512, 512), &framebuffer, 3 * 512)
      .unwrap();

    canvas.copy(&texture, None, Some(Rect::new(0, 0, 512, 512)))?;
    canvas.present();

    ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
  }

  Ok(())
}

fn draw_rectangle(
  buffer: &mut [u8],
  img_w: usize,
  img_h: usize,
  x: usize,
  y: usize,
  w: usize,
  h: usize,
  color: sdl2::pixels::Color,
) {
  for i in 0..w {
    for j in 0..h {
      let cx = x + i;
      let cy = y + j;

      let offset = (cx + cy * img_w) * 3; // SDL2, 24-bit colour specific tweak: multiply original offset b 3.

      buffer[offset] = color.r;
      buffer[offset + 1] = color.g;
      buffer[offset + 2] = color.b;
    }
  }
}
