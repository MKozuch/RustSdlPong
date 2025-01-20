#![allow(dead_code)]
#[allow(unused)]

pub mod draw_primitives {
    use std::ops::Mul;

    use sdl2::rect::Rect;
    use sdl2::render::Canvas;
    use sdl2::render::Texture;
    use sdl2::video::Window;
    use sdl2::render::RenderTarget;
    use sdl2::rect::{FPoint, Point};


    pub fn draw_polygon_regular<T: RenderTarget>(canvas: &mut sdl2::render::Canvas<T>, center: FPoint, edges: u32, size: f32, angle_offset: f32 ) -> Result<(), String> {
        let mut points: Vec<FPoint> = Vec::new();
        let r = size / 2.0;
        for i in 0..edges {
            let fi = (std::f32::consts::PI * 2.0 / edges as f32 * i as f32) + angle_offset;
            points.push(FPoint::new(
                center.x + (r * fi.cos()),
                center.y + (r * fi.sin()),
            ));
        }
        points.push(points[0]);
        canvas.draw_flines(points.as_slice())?;
        Ok(())
    }
    
    pub fn draw_gradient_rect<T: RenderTarget>(canvas: &mut sdl2::render::Canvas<T>, gradient_base_texture: &mut Texture, dest_rect: Rect, angle: f64, color1: sdl2::pixels::Color, color2: sdl2::pixels::Color) -> Result<(), String> {
        gradient_base_texture.set_blend_mode(sdl2::render::BlendMode::Blend);
        gradient_base_texture.set_alpha_mod(255);
        
        gradient_base_texture.set_color_mod(color1.r, color1.g, color1.b);

        canvas.set_clip_rect(dest_rect);
        canvas.copy_ex(
            &gradient_base_texture, 
            None, 
            None,
            angle, 
            None, 
            false, 
            false)?;

        gradient_base_texture.set_color_mod(color2.r, color2.g, color2.b);

        canvas.copy_ex(
            &gradient_base_texture, 
            None, 
            None, 
            angle, 
            None, 
            true, 
            true)?;

        canvas.set_clip_rect(None);
        Ok(())
    }
}