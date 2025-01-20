#![allow(dead_code)]
#[allow(unused)]

pub mod tick_controller {
    pub struct TickController {
        target_fps: u32,
        frame_count: u64,
        current_tick_actual: std::time::Instant,
        scheduled_tick: std::time::Instant,
        previous_tick_actual: std::time::Instant,
        tick_interval: std::time::Duration,
        last_fps_check : std::time::Instant,
        paused: bool,
    } 

    // "Something is wrong with, the calculated fps is too high"
    impl TickController {

        pub fn pause(&mut self) {
            self.paused = true;
        }

        pub fn resume(&mut self) {
            self.paused = false;
        }

        pub fn from_target_fps(target_fps: u32) -> Self {
            let current_tick = std::time::Instant::now();
            let next_tick = current_tick;
            let previous_tick = current_tick;
            let tick_interval = std::time::Duration::from_secs_f32(1.0 / target_fps as f32);
            let frame_count = 0;
            let last_fps_check = std::time::Instant::now();

            TickController {
                target_fps,
                frame_count,
                current_tick_actual: current_tick,
                scheduled_tick: next_tick,
                previous_tick_actual: previous_tick,
                tick_interval,
                last_fps_check,
                paused: false,
            }
        }

        pub fn wait_for_next_tick(&mut self) {
            self.frame_count += 1;
            if self.last_fps_check.elapsed().as_secs_f32() > 1.0 {
                println!("FPS: {}", self.fps_check());
            }

            let now: std::time::Instant = std::time::Instant::now();
            let sleep_duration = self.scheduled_tick.saturating_duration_since(now);
            let busy_duration = now.saturating_duration_since(self.current_tick_actual);

            if sleep_duration.as_micros() > 0 {
                while std::time::Instant::now() < self.scheduled_tick {
                    ();
                }
                //std::thread::sleep(sleep_duration);
            }
            else {
                let missed = now.saturating_duration_since(self.scheduled_tick);
                //println!("Missed frame by: {}ms", missed.as_millis());
            }

            // println!("Busy: {}ms\tSleeping: {}ms", busy_duration.as_millis(), sleep_duration.as_millis());            
            
            // onset of next tick
            self.scheduled_tick += self.tick_interval;  
            while self.scheduled_tick < std::time::Instant::now() {
                self.scheduled_tick += self.tick_interval;
                // println!("Rescheduled tick");
            }

            self.previous_tick_actual = self.current_tick_actual;
            self.current_tick_actual = std::time::Instant::now();

        }

        pub fn elapsed_since_last_tick(&self) -> std::time::Duration {
            self.previous_tick_actual.elapsed()
        }

        // pub fn time_to_next_tick(&self) -> std::time::Duration {
        //     self.scheduled_tick.saturating_duration_since(std::time::Instant::now())
        // }

        // pub fn delta_t(&self) -> std::time::Duration {
        //     self.tick_interval
        // }

        pub fn fps_check(&mut self) -> f32 {
            let elapsed = self.last_fps_check.elapsed().as_secs_f32();
            let fps = self.frame_count as f32 / elapsed;
            self.frame_count = 0;
            self.last_fps_check = std::time::Instant::now();
            return fps;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tick_controller::tick_controller::TickController;

    #[test]
    fn fps_test() {
        let target_fps = 60;
        let mut tick_controller = TickController::from_target_fps(target_fps);
    
        let start = std::time::Instant::now();
        let mut frame_count = 0;
        
        while start.elapsed().as_secs_f32() < 1.0 {
            tick_controller.wait_for_next_tick();
            std::thread::sleep(std::time::Duration::from_millis(1));
            frame_count += 1;
        }
        println!("Frames: {}", frame_count);
        assert_eq!(frame_count >= target_fps - 1 && frame_count <= target_fps + 1, true);
    }

    #[test]
    fn time_sleep() {
        let start = std::time::Instant::now();

        for _ in 0..100 {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let elapsed = start.elapsed();
        println!("Elapsed: {}s", elapsed.as_secs_f32());        
    }

    #[test]
    fn time_sleep2() {
        let start = std::time::Instant::now();
        let mut count = 0;

        while start.elapsed().as_secs_f32() < 1.0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
            count += 1;
        }

        let elapsed = start.elapsed();
        println!("Elapsed: {}s", elapsed.as_secs_f32());       
        println!("Count: {}", count); 
    }
}

struct FrameStats {
    frame_count: u64,
    skipped_frames: u64,
}