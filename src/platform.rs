extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Texture;

use std::time::Duration;

struct Platform {

}

impl Platform {
    pub fn new(title: String, window_width: u16, window_height: u16, texture_width: u16, texture_heigh: u16) {
        let number = 8;
        let sdl_context = sdl2::init();
        let sdl_context = match sdl_context {
            Ok(sdl) => sdl,
            Err(e) => panic!("Error initializing Sdl for rendering. Error message: {}", e),
        };

        let video_subsystem = sdl_context.video();
        let video_subsystem = match video_subsystem {
            Ok(video_subsystem) => video_subsystem,
            Err(e) => panic!("Error initializing Sdl video subsystem. Error message: {}", e),
        };

        let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
            .position_centered()
            .build()
            .unwrap();

            let mut canvas = window.into_canvas().build().unwrap();
    }
}
