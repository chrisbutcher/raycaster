extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

use std::time::Duration;

use std::fs;

pub fn main() -> Result<(), String> {
  let sdl_context = sdl2::init()?;
  let video_subsystem = sdl_context.video()?;

  const WIN_W: usize = 1024;
  const WIN_H: usize = 512;

  let window = video_subsystem
    .window("raycaster", WIN_W as u32, WIN_H as u32)
    .position_centered()
    .opengl()
    .build()
    .map_err(|e| e.to_string())?;

  let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
  let texture_creator = canvas.texture_creator();

  let mut texture = texture_creator
    .create_texture_streaming(PixelFormatEnum::RGB24, WIN_W as u32, WIN_H as u32)
    .map_err(|e| e.to_string())?;

  // Raycaster code port, from https://github.com/ssloy/tinyraycaster/wiki

  let map_w = 16;
  let map_h = 16;

  let mut framebuffer: [u8; WIN_H * WIN_W * 3] = [0x33; WIN_H * WIN_W * 3];

  canvas.set_draw_color(sdl2::pixels::Color::RGB(33, 33, 33));

  let map = fs::read_to_string("map.txt").unwrap();
  let map: Vec<char> = map.chars().filter(|ch| *ch != '\n').collect();

  let player_x = 3.456_f64;
  let player_y = 2.345_f64;
  let mut player_a = 1.523_f64;
  const FOV: f64 = std::f64::consts::PI / 3.0;

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

    // clear framebuffer
    for elem in framebuffer.iter_mut() {
      *elem = 0x33;
    }

    let rect_w = WIN_W / (map_w * 2);
    let rect_h = WIN_H / map_h;

    // Draw map
    for j in 0..map_h {
      for i in 0..map_w {
        let wall_tile = map[i + j * map_w];

        if wall_tile == ' ' {
          continue;
        }

        let rect_x = i * rect_w;
        let rect_y = j * rect_h;

        draw_rectangle(
          &mut framebuffer,
          WIN_W,
          WIN_H,
          rect_x,
          rect_y,
          rect_w,
          rect_h,
          wall_tile_to_color(wall_tile),
        );
      }
    }

    // Draw player
    draw_rectangle(
      &mut framebuffer,
      WIN_W,
      WIN_H,
      (player_x * rect_w as f64) as usize,
      (player_y * rect_h as f64) as usize,
      5,
      5,
      sdl2::pixels::Color::RGB(255, 255, 255),
    );

    player_a += 2.0 * std::f64::consts::PI / 360.0;

    // Casting 512 rays
    for i in 0..WIN_W / 2 {
      let angle = player_a - FOV / 2.0 + FOV * i as f64 / (WIN_W / 2) as f64;

      let mut t = 0.0_f64;
      loop {
        let cx = player_x + t * angle.cos();
        let cy = player_y + t * angle.sin();

        let pix_x = (cx * rect_w as f64) as usize;
        let pix_y = (cy * rect_h as f64) as usize;

        let offset = (pix_x + pix_y * WIN_W) * 3;

        framebuffer[offset] = 160;
        framebuffer[offset + 1] = 160;
        framebuffer[offset + 2] = 160;

        let wall_hit = map[cx as usize + cy as usize * map_w];

        if wall_hit != ' ' {
          let column_height = (WIN_H as f64 / (t * (angle - player_a).cos())) as usize;

          draw_rectangle(
            &mut framebuffer,
            WIN_W,
            WIN_H,
            WIN_W / 2 + i,
            WIN_H / 2 - column_height / 2,
            1,
            column_height,
            wall_tile_to_color(wall_hit),
          );
          break;
        }

        t = t + 0.01;

        if t > 20.0 {
          break;
        }
      }
    }

    // Game loop END

    texture
      .update(Rect::new(0, 0, WIN_W as u32, WIN_H as u32), &framebuffer, 3 * WIN_W)
      .unwrap();

    canvas.copy(&texture, None, Some(Rect::new(0, 0, WIN_W as u32, WIN_H as u32)))?;
    canvas.present();

    ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
  }

  Ok(())
}

fn draw_rectangle(
  framebuffer: &mut [u8],
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

      if cx >= img_w || cy >= img_h {
        continue; // no need to check for negative values (unsigned variables)
      };

      let offset = (cx + cy * img_w) * 3; // SDL2, 24-bit colour specific tweak: multiply original offset b 3.

      framebuffer[offset] = color.r;
      framebuffer[offset + 1] = color.g;
      framebuffer[offset + 2] = color.b;
    }
  }
}

fn wall_tile_to_color(wall_char: char) -> sdl2::pixels::Color {
  match wall_char {
    '2' => sdl2::pixels::Color::RGB(0, 255, 255),
    '1' => sdl2::pixels::Color::RGB(255, 0, 255),
    _ => sdl2::pixels::Color::RGB(255, 255, 0),
  }
}
