#![allow(unused_imports)]
#![allow(unused_variables)]


extern crate rand;
extern crate sdl2;

use peng::{BallPhysics, GameController, PlayerPaddleController, PangGameController, PangGameState, PaddleAIController};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::libc::can_err_mask_t;
use sdl2::pixels::Color;
use sdl2::rect::{FPoint, FRect, Rect};
use sdl2::render::TextureCreator;
use std::time;

mod tick_controller;
use tick_controller::tick_controller::TickController;

mod draw_primitives;
use draw_primitives::draw_primitives::*;

pub fn main() -> Result<(), String> {
    println!("Hello, world!");
    println!("SDL2 Version: {}", sdl2::version::version());

    let window_size = (
        800,
        600,
    );

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "rust-sdl2 demo: Video",
            window_size.0,
            window_size.1,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas: sdl2::render::Canvas<sdl2::video::Window> = window
                                                                    .into_canvas()
                                                                    .accelerated()
                                                                    .present_vsync()
                                                                    .build().map_err(|e| e.to_string())?;

    //canvas.set_blend_mode(sdl2::render::BlendMode::Add);
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    let canvas_viewport = canvas.viewport();

    let use_filtering = true;
    if use_filtering {
        unsafe { sdl2::sys::SDL_SetHint(sdl2::sys::SDL_HINT_RENDER_SCALE_QUALITY.as_ptr() as *const i8, b"1\0".as_ptr() as *const i8) };
    }
    else{
        unsafe { sdl2::sys::SDL_SetHint(sdl2::sys::SDL_HINT_RENDER_SCALE_QUALITY.as_ptr() as *const i8, b"0\0".as_ptr() as *const i8) };
    }

    let texture_builder = canvas.texture_creator();

    let mut gradient_base_texture = texture_builder.create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGBA32, 2, 1).unwrap();
    gradient_base_texture.set_blend_mode(sdl2::render::BlendMode::Blend);

    gradient_base_texture.with_lock(None, |buffer: &mut [u8], pitch: usize|{
        buffer[0] = 255;
        buffer[1] = 255;
        buffer[2] = 255;
        buffer[3] = 255;

        buffer[4] = 0;
        buffer[5] = 0;
        buffer[6] = 0;
        buffer[7] = 0;

        // red to green
        // buffer[0] = 255;
        // buffer[1] = 0;
        // buffer[2] = 0;
        // buffer[3] = 255;

        // buffer[4] = 0;
        // buffer[5] = 255;
        // buffer[6] = 0;
        // buffer[7] = 255;
    })?;

    gradient_base_texture.set_color_mod(255, 0, 0);

    let start = time::SystemTime::now();

    loop{
        canvas.clear();

       let asdf = (time::SystemTime::now().duration_since(start).unwrap().as_secs_f32() / 5.0).fract() as f32;
       let alpha = ((asdf * 2.0 * std::f32::consts::PI).sin() * 255.0/2.0 + 255.0/2.0 ) as u8;

        gradient_base_texture.set_alpha_mod(alpha);

        canvas.copy(&gradient_base_texture, None, Rect::new(0, 0, canvas_viewport.width()/2, canvas_viewport.height()))?;
        canvas.copy_ex(
            &gradient_base_texture, 
            None, 
            Rect::new((canvas_viewport.width()/2) as i32, 0, canvas_viewport.width()/2, canvas_viewport.height()), 
            0.0, 
            None, 
            true, 
            true)?;

        canvas.present();

        break;
    }

    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;

    let initial_velocity = 250.0;
    let initial_angle = rand::random::<f32>() * std::f32::consts::PI * 2.0;

    let ball_physics = BallPhysics {
        horizontal_acc: 0.0,
        vertical_acc: 0.0,
        restitution_factor: 1.1,
        restitution_angle_variance: 0.0,
        inherited_velocity: 0.0,
        max_velocity: 1000.0,
    };

    let ball = peng::Ball {
        size: 50,
        pos: FPoint::new(
            canvas_viewport.center().x() as f32,
            canvas_viewport.center().y() as f32,
        ),
        velocity: FPoint::new(
            initial_velocity * initial_angle.cos(),
            initial_velocity * initial_angle.sin(),
        ),
        physics: ball_physics.clone(),
    };

    let paddle_offset = 20.0;
    let paddle_size = FPoint::new(20.0, 100.0);

    let paddle_left = peng::Paddle {
        size: paddle_size,
        pos: FPoint::new( canvas.viewport().left() as f32 + paddle_offset, canvas.viewport().center().y() as f32),
        velocity: FPoint::new(0.0, 0.0),
        acceleration: FPoint::new(0.0, 0.0),
        movement_speed: 500.0,
    };

    let paddle_right = peng::Paddle {
        size: paddle_size,
        pos: FPoint::new(  canvas.viewport().right() as f32 - paddle_offset, canvas.viewport().center().y() as f32),
        velocity: FPoint::new(0.0, 0.0),
        acceleration: FPoint::new(0.0, 0.0),
        movement_speed: 500.0,
    };

    let target_fps = 600;
    let mut tick_controller = TickController::from_target_fps(target_fps);

    let mut game_state = PangGameState {
        ball: ball,
        paddle_left: paddle_left,
        paddle_right: paddle_right,
        canvas: FRect::new(
            canvas_viewport.x() as f32,
            canvas_viewport.y() as f32, 
            canvas_viewport.width() as f32, 
            canvas_viewport.height() as f32,
        ),
    };

    let mut game_state_controller = PangGameController{
        paddle_controller_left: Box::new(PaddleAIController::new()),
        //paddle_controller_left: PlayerPaddleController::new(Keycode::W, Keycode::S),
        paddle_controller_right: Box::new(PaddleAIController::new()),
    };

    'running: loop {
        game_state = game_state_controller.update(&game_state, time::Instant::now(), tick_controller.elapsed_since_last_tick(), Event::Unknown { timestamp: 0, type_: 0 })?;

        for event in event_pump.poll_iter(){
            match &event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {keycode: Some(Keycode::R), ..} => {
                    game_state.ball.pos = FPoint::new(
                        canvas.viewport().center().x() as f32,
                        canvas.viewport().center().y() as f32,
                    );
                    game_state.ball.velocity = FPoint::new(
                        initial_velocity * initial_angle.cos(),
                        initial_velocity * initial_angle.sin(),
                    );
                },
                _ => {game_state_controller.update(&mut game_state, time::Instant::now(), time::Duration::from_micros(0), event)?;}
            }
        }

        canvas.set_draw_color(sdl2::pixels::Color::BLACK);
        canvas.clear();

        let rect = Rect::from_center(canvas_viewport.center(), 400, 300 );
        draw_gradient_rect(&mut canvas, &mut gradient_base_texture, rect, 45.0, sdl2::pixels::Color::BLUE, sdl2::pixels::Color::RED)?;
        game_state_controller.draw(&game_state, &mut canvas)?;


        canvas.present();

        game_state.canvas = FRect::new(
            canvas.viewport().x() as f32,
            canvas.viewport().y() as f32, 
            canvas.viewport().width() as f32, 
            canvas.viewport().height() as f32,
        );

        tick_controller.wait_for_next_tick();
    }

    return Ok(());
}
