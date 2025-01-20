#![allow(dead_code)]
#[allow(unused)]

mod draw_primitives;
use draw_primitives::draw_primitives::draw_polygon_regular;
use sdl2::sys::SDL_GetTicks;

use std::f32::consts::PI;
use std::time;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use sdl2::event::Event;
use sdl2::rect::FPoint;
use sdl2::rect::FRect;
use sdl2::rect::Point;
use sdl2::render::RenderTarget;
use sdl2::keyboard::Keycode;

#[derive(Copy, Clone)]
pub struct Ball {
    pub size: i32,
    pub pos: FPoint,
    pub velocity: FPoint,
    pub physics: BallPhysics,
}

#[derive(Copy, Clone)]
pub struct Paddle {
    pub size: FPoint,
    pub pos: FPoint,
    pub velocity: FPoint,
    pub acceleration: FPoint,
    pub movement_speed: f32, 
}

#[derive(Copy, Clone)]
pub struct PlayField {
    pub rect: FRect,
}

impl PlayField{
    pub fn from_rect(rect: FRect) -> PlayField {
        PlayField {
            rect: rect,
        }
    }
}

pub trait Drawable {
    fn draw<T: RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>) -> Result<(), String>;
}

impl Drawable for Ball {
    fn draw<T: RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>) -> Result<(), String> {

        canvas.set_draw_color(sdl2::pixels::Color::GREEN);

        enum DrawImp{
            Dots,
            Polygon,
        }

        let draw_dots = |canvas: &mut sdl2::render::Canvas<T>| -> Result<(), String> {
            let mut points: Vec<Point> = Vec::new();
            let resolution = 4;
            for x in (-self.size..self.size).step_by(resolution) {
                for y in (-self.size/2..self.size/2).step_by(resolution){
                    let mut point_x  = self.pos.x as i32 + x;
                    let mut point_y = self.pos.y as i32 + y;
                    point_x -= point_x % resolution as i32;
                    point_y -= point_y % resolution as i32;
    
                    points.push(
                        Point::new(
                            point_x,
                            point_y,
                        ));
                }
            }
            points.retain(|point| (self.pos.x() - point.x() as f32).hypot(self.pos.y() - point.y() as f32) <= (self.size as f32) / 2 as f32);
            canvas.draw_points(points.as_slice())?;
            Ok(())
        };

        let draw_poly = |canvas: &mut sdl2::render::Canvas<T>|{
            let asdf = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_millis() as f32 / 1000.0;
            let angle_offset = asdf * 2.0 * PI;

            draw_polygon_regular(canvas, self.pos, 7, self.size as f32, angle_offset)
        };

        let draw_imp = DrawImp::Polygon;

        match draw_imp {
            DrawImp::Dots => draw_dots(canvas),
            DrawImp::Polygon => draw_poly(canvas),
        }?;

        Ok(())
    }
}

impl Drawable for Paddle {
    fn draw<T: RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>) -> Result<(), String> {
        canvas.set_draw_color(sdl2::pixels::Color::GREEN);
        canvas.draw_frect(FRect::from_center(self.pos, self.size.x(), self.size.y()))?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct BallPhysics {
    pub horizontal_acc: f32,
    pub vertical_acc: f32,
    pub restitution_factor: f32,
    pub restitution_angle_variance: f32,
    pub inherited_velocity: f32,
    pub max_velocity: f32,
}

// todo: Unify the Kinematic trait for Ball and Paddle
pub trait Kinematic {
    fn update(&mut self, tick_interval: std::time::Duration);
    fn set_velocity(&mut self, velocity: FPoint);
    fn translate(&mut self, translation: FPoint);
}

impl Kinematic for Ball {
    fn update(&mut self, tick_interval: std::time::Duration) {
        self.pos = FPoint::new(
            self.pos.x() + self.velocity.x() * tick_interval.as_secs_f32() + 0.5 * self.physics.horizontal_acc * tick_interval.as_secs_f32().powi(2) as f32,
            self.pos.y() + self.velocity.y() * tick_interval.as_secs_f32() + 0.5 * self.physics.vertical_acc * tick_interval.as_secs_f32().powi(2) as f32,
        );

        self.velocity = FPoint::new(
            self.velocity.x() + self.physics.horizontal_acc * tick_interval.as_secs_f32(),
            self.velocity.y() + self.physics.vertical_acc * tick_interval.as_secs_f32(),
        );
    }

    fn set_velocity(&mut self, velocity: FPoint) {
        self.velocity = velocity;
    }

    fn translate(&mut self, translation: FPoint) {
        self.pos = FPoint::new(self.pos.x() + translation.x(), self.pos.y() + translation.y());
    }
}

impl Kinematic for Paddle {
    fn update(&mut self, tick_interval: std::time::Duration) {
        self.pos = FPoint::new(
            self.pos.x() + self.velocity.x() * tick_interval.as_secs_f32() + 0.5 * self.acceleration.x() * tick_interval.as_secs_f32().powi(2) as f32,
            self.pos.y() + self.velocity.y() * tick_interval.as_secs_f32() + 0.5 * self.acceleration.y() * tick_interval.as_secs_f32().powi(2) as f32,
        );

        self.velocity = FPoint::new(
            self.velocity.x() + self.acceleration.x() * tick_interval.as_secs_f32(),
            self.velocity.y() + self.acceleration.y() * tick_interval.as_secs_f32(),
        );
    }

    fn set_velocity(&mut self, velocity: FPoint) {
        self.velocity = velocity;
    }

    fn translate(&mut self, translation: FPoint) {
        self.pos = FPoint::new(self.pos.x() + translation.x(), self.pos.y() + translation.y());
    }
}

pub trait BetterPoint<PointType> {
    fn magnitude(&self) -> f32;
    fn normalize(&self) -> PointType;
    fn angle(&self) -> f32;
}

impl BetterPoint<FPoint> for FPoint {
    fn magnitude(&self) -> f32 {
        return self.x().hypot(self.y())
    }

    fn normalize(&self) -> FPoint {
        let magnitude = self.magnitude();
        FPoint::new(self.x() / magnitude, self.y() / magnitude)
    }

    fn angle(&self) -> f32 {
        self.y().atan2(self.x())
    }
}


pub trait Collider {
    fn collider(&self) -> FRect;
}

impl Collider for Ball {
    fn collider(&self) -> FRect {
        FRect::from_center(self.pos, self.size as f32, self.size as f32)
    }
}

impl Collider for Paddle {
    fn collider(&self) -> FRect {
        FRect::from_center(self.pos, self.size.x(), self.size.y())
    }
}

impl Collider for PlayField {
    fn collider(&self) -> FRect {
        self.rect
    }
}

// todo: Join the Collide trait with the Collider trait
pub trait Collide<T: Collider>: Collider {
    fn collide(&mut self, other: &T);
}

// somehning is off here, the velocity vector after has wrong direction
impl Collide<Paddle> for Ball{
    fn collide(&mut self, other: &Paddle) {

        let intersection = self.collider().intersection(other.collider());
        if intersection.is_none() {
            return;
        }

        let intersection = intersection.unwrap();
        
        let velocity_vector = self.velocity.normalize();
        while self.collider().has_intersection(other.collider()) {
          self.translate(-velocity_vector );
        }

        if intersection.width() > intersection.height() {
            self.velocity = FPoint::new(self.velocity.x(), -self.velocity.y());
        }
        else if intersection.height() >  intersection.width() {
            self.velocity = FPoint::new(-self.velocity.x(), self.velocity.y());
        }
        else {
            self.velocity = FPoint::new(-self.velocity.x(), -self.velocity.y());
        }

        let new_velocity_angle = self.velocity.angle() + (rand::random::<f32>() * std::f32::consts::PI / 2.0 - std::f32::consts::PI / 4.0) * self.physics.restitution_angle_variance;
        let new_velocity_magnitude = (self.velocity.magnitude() * self.physics.restitution_factor).clamp(0.0, self.physics.max_velocity);

        self.velocity = FPoint::new(
            new_velocity_magnitude * new_velocity_angle.cos(),
            new_velocity_magnitude * new_velocity_angle.sin(),
        );
    }
}

impl Collide<Ball> for Ball{
    fn collide(&mut self, other: &Ball) {
        let ball_collider = self.collider();
        let other_collider = other.collider();

        if ball_collider.left() < other_collider.right() && ball_collider.right() > other_collider.left() && ball_collider.top() < other_collider.bottom() && ball_collider.bottom() > other_collider.top() {
            self.velocity = FPoint::new(-self.velocity.x(), -self.velocity.y());
        }
    }
}

impl Collide<PlayField> for Ball{
    fn collide(&mut self, other: &PlayField) {
        let ball_collider: FRect = self.collider();

        let union = other.rect.union(ball_collider);
        if union != other.rect {
            println!("bump!, {}", self.velocity.magnitude());        

            if union.width() > other.rect.width() {
                self.velocity = FPoint::new(-1.0 * self.velocity.x(), self.velocity.y());
            }
            if union.height() > other.rect.height() {
                self.velocity = FPoint::new(self.velocity.x(), self.velocity.y() * -1.0);
            }

            let window_offset = 1.0;
            self.pos = FPoint::new(
                self.pos.x().clamp(
                    other.rect.left() + self.size as f32 / 2.0 + window_offset,
                    other.rect.right() - self.size as f32 / 2.0 - window_offset,
                ),
                self.pos.y().clamp(
                    other.rect.top() + self.size as f32 / 2.0 + window_offset,
                    other.rect.bottom() - self.size as f32 / 2.0 - window_offset,
                ),
            );
        }
    }
}

impl Collide<PlayField> for Paddle{
    fn collide(&mut self, other: &PlayField) {
        self.pos = FPoint::new(
            self.pos.x().clamp(
                other.rect.left() + self.size.x() / 2.0,
                other.rect.right() - self.size.x() / 2.0,
            ),
            self.pos.y().clamp(
                other.rect.top() + self.size.y() / 2.0,
                other.rect.bottom() - self.size.y() / 2.0,
            ),
        );
    }
}


#[derive(Copy, Clone)]
pub struct PangGameState {
    pub ball: Ball,
    pub paddle_left: Paddle,
    pub paddle_right: Paddle,
    pub canvas: FRect,
}

pub trait GameController<State>{
    fn update(&mut self, game_state: & State, next_tick: time::Instant, delta_t: time::Duration, event: Event) -> Result<State, String>;
    fn draw<T: RenderTarget>(&self, game_state: &State, canvas: &mut sdl2::render::Canvas<T>) -> Result<(), String>;
}

pub struct PangGameController{
    pub paddle_controller_left: Box<dyn PaddleController>, 
    pub paddle_controller_right: Box<dyn PaddleController>,
}

impl GameController<PangGameState>  for PangGameController{
    fn update(&mut self, game_state_ref: &PangGameState, _next_tick: time::Instant, delta_t: time::Duration, event: Event) -> Result<PangGameState, String> {
        
        // todo: remove clone and buikd the state in a more functional way  
        let mut game_state = game_state_ref.clone();
        let ball = &mut game_state.ball;
        let paddle_left = &mut game_state.paddle_left;
        let paddle_right = &mut game_state.paddle_right;

        paddle_left.pos.x = game_state.canvas.left() + 20.0;
        paddle_right.pos.x = game_state.canvas.right() - 20.0;

        self.paddle_controller_left.update_paddle(game_state_ref, &event, paddle_left);
        self.paddle_controller_right.update_paddle(game_state_ref, &event, paddle_right);

        ball.update(delta_t);
        paddle_left.update(delta_t);
        paddle_right.update(delta_t);

        let play_field = PlayField::from_rect(game_state.canvas);

        ball.collide(paddle_left);
        ball.collide(paddle_right);
        ball.collide(&play_field);
        
        paddle_left.collide(&play_field);
        paddle_right.collide(&play_field);

        Ok(game_state)
    }
    
    fn draw<T: RenderTarget>(&self, game_state: &PangGameState, canvas: &mut sdl2::render::Canvas<T>) -> Result<(), String> {
        game_state.ball.draw(canvas)?;
        game_state.paddle_left.draw(canvas)?;
        game_state.paddle_right.draw(canvas)?;
        
        Ok(())
    }
}


pub trait PaddleController{
    fn update_paddle(&mut self, game_state: &PangGameState, event: &Event, paddle: &mut Paddle);
}


pub struct PlayerPaddleController{
    down_btn_pressed: bool,
    up_btn_pressed: bool,

    keycode_up: Keycode,
    keycode_down: Keycode,
}

impl PlayerPaddleController {
    pub fn new(keycode_up: Keycode, keycode_down: Keycode) -> PlayerPaddleController {
        PlayerPaddleController {
            down_btn_pressed: false,
            up_btn_pressed: false,
            keycode_up: keycode_up,
            keycode_down: keycode_down,
        }
    }

    pub fn pressed_down(&mut self) {
        self.down_btn_pressed = true;    
    }
    pub fn pressed_up(&mut self) {
        self.up_btn_pressed = true;
    }
    pub fn released_down(&mut self) {
        self.down_btn_pressed = false;
    }
    pub fn released_up(&mut self) {
        self.up_btn_pressed = false; 
    }
}

impl PaddleController for PlayerPaddleController {
    fn update_paddle(&mut self, _game_state: &PangGameState, event: &Event, paddle: &mut Paddle) {

        match event {
            Event::KeyDown { keycode: Some(keycode), .. } => {
                match keycode {
                    keycode if *keycode == self.keycode_up => {
                        self.pressed_up();
                    }
                    keycode if *keycode == self.keycode_down => {
                        self.pressed_down();
                    }
                    _ => {}
                }
            }
            Event::KeyUp { keycode: Some(keycode), .. } => {
                match keycode {
                    keycode if *keycode == self.keycode_up => {
                        self.released_up();
                    }
                    keycode if *keycode == self.keycode_down => {
                        self.released_down();
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        if self.down_btn_pressed == self.up_btn_pressed {
            paddle.velocity = FPoint::new(0.0, 0.0);
        }
        else if self.down_btn_pressed {
            paddle.velocity = FPoint::new(0.0, 1.0) * paddle.movement_speed;
        }
        else if self.up_btn_pressed {
            paddle.velocity = FPoint::new(0.0, -1.0) * paddle.movement_speed;
        }
    }
}

pub struct PaddleAIController{
}

impl PaddleAIController {
    pub fn new() -> PaddleAIController {
        PaddleAIController {
        }
    }
}

impl PaddleController for PaddleAIController {
    fn update_paddle(&mut self, game_state: &PangGameState, _event: &Event, paddle: &mut Paddle) {

        let mut target_speed_y: f32;
        let target_y: f32;
        let is_ball_moving_towards_paddle = game_state.ball.velocity.x().signum() == (paddle.pos.x()-game_state.ball.pos.x()).signum();
        

        if is_ball_moving_towards_paddle {
            target_y = game_state.ball.pos.y();
            target_speed_y = game_state.ball.velocity.y();
        }
        else{
            target_y = game_state.canvas.center().y();
            target_speed_y = 0.0;
        }

        let on_target = (paddle.pos.y() - target_y).abs() < paddle.size.y() / 4.0; 

        if on_target {
            paddle.velocity = FPoint::new(0.0, target_speed_y);
        }
        else{
            paddle.velocity = FPoint::new(0.0, (target_y - paddle.pos.y()).signum() * paddle.movement_speed);
        }
    }
}


trait PaddleMover {
    fn move_up(&mut self, paddle: &mut Paddle){
        paddle.velocity = FPoint::new(0.0, -1.0) * paddle.movement_speed;
    }

    fn move_down(&mut self, paddle: &mut Paddle){
        paddle.velocity = FPoint::new(0.0, 1.0) * paddle.movement_speed;
    }

    fn stop(&mut self, paddle: &mut Paddle){
        paddle.velocity = FPoint::new(0.0, 0.0);
    }
}
