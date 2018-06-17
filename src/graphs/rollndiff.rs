use definitions::*;
use ggez::graphics::*;
use ggez::*;

use std::collections::VecDeque;
use std::time::Duration;

pub struct RollGraphData {}

fn _linearize(
    tyre_array: &TyresArray<f32>,
    width: f32,
    height: f32,
    mult: f32,
) -> ([Point2; 2], [Point2; 2]) {
    let center_f = Point2::new(0f32, -height / 2f32);
    let center_r = Point2::new(0f32, height / 2f32);

    let (fl, fr) = (
        tyre_array.data[Tyre::TyreFrontLeft as usize],
        tyre_array.data[Tyre::TyreFrontRight as usize],
    );
    let (rl, rr) = (
        tyre_array.data[Tyre::TyreRearLeft as usize],
        tyre_array.data[Tyre::TyreRearRight as usize],
    );

    (
        [
            Point2::new(center_f.x - width, center_f.y + fl * mult),
            Point2::new(center_f.x + width, center_f.y + fr * mult),
        ],
        [
            Point2::new(center_r.x - width, center_r.y + rl * mult),
            Point2::new(center_r.x + width, center_r.y + rr * mult),
        ],
    )
}

pub struct RakeGraphData {
    current_time: Duration,
    current_front: f32,
    current_rear: f32,
    start_time: Duration,
    pub max_height: f32,
    mesh_front: VecDeque<(Duration, f32)>,
    mesh_rear: VecDeque<(Duration, f32)>,
}

impl RakeGraphData {
    pub fn new() -> RakeGraphData {
        let dur = Duration::from_secs(0);
        RakeGraphData {
            current_time: dur,
            current_front: 0f32,
            current_rear: 0f32,
            start_time: dur,
            max_height: 0f32,
            mesh_front: VecDeque::new(),
            mesh_rear: VecDeque::new(),
        }
    }

    pub fn add(&mut self, front_avg: f32, rear_avg: f32, time_dur: Duration) {
        let start_time = time_dur.checked_sub(Duration::from_secs(60));

        self.mesh_front.push_back((self.current_time, front_avg));
        self.mesh_rear.push_back((self.current_time, rear_avg));

        self.current_front = front_avg;
        self.current_rear = rear_avg;

        self.max_height = self.max_height.max(front_avg).max(rear_avg);
        self.current_time = time_dur;

        if let Some(start_time) = start_time {
            let drop_front = self
                .mesh_front
                .iter()
                .take_while(|(t, _)| *t < start_time)
                .count();

            let drop_rear = self
                .mesh_rear
                .iter()
                .take_while(|(t, _)| *t < start_time)
                .count();
            {
                self.mesh_rear.drain(0..drop_rear).next();
                self.mesh_front.drain(0..drop_front).next();
            }

            if let Some((st_f, _)) = self.mesh_front.pop_front() {
                self.start_time = st_f;
            }
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, screen_size: &Point2) -> GameResult<()> {
        let end = self.current_time.clone();

        let max = self.max_height * 1.2;

        let x_start = screen_size.x * 0.6;
        let x_end = screen_size.x - (screen_size.x - x_start) * 0.2;
        let y_start = screen_size.y * 0.525;
        let y_end = screen_size.y;

        let y_coefficient = |rh: &f32| -> f32 { y_start + ((y_end - y_start) * (rh / max)) };

        let x_coefficient = |dur: &Duration| -> f32 {
            let to_end = end - *dur;
            let coeff =
                (to_end.as_secs() as f32 * 1000f32 + to_end.subsec_millis() as f32) / 60000f32;

            x_end - (x_end - x_start) * coeff
        };

        if self.mesh_front.len() > 1 {
            // let start = Point2::new(x_start, y_coefficient(&self.start_diff));
            let end = Point2::new(
                x_coefficient(&self.current_time),
                y_coefficient(&self.current_front),
            );

            let mut line_front: Vec<Point2> = self
                .mesh_front
                .iter()
                .map(|tup| Point2::new(x_coefficient(&tup.0), y_coefficient(&tup.1)))
                .collect();

            line_front.push(end);

            let mesh = Mesh::new_line(ctx, &line_front, 2f32)?;

            graphics::set_color(ctx, Color::from_rgb(173, 255, 47))?;
            mesh.draw(ctx, Point2::new(0f32, 0f32), 0f32)?;
        }

        if self.mesh_rear.len() > 1 {
            let end = Point2::new(
                x_coefficient(&self.current_time),
                y_coefficient(&self.current_rear),
            );

            let mut line_rear: Vec<Point2> = self
                .mesh_rear
                .iter()
                .map(|(x, y)| Point2::new(x_coefficient(x), y_coefficient(y)))
                .collect();

            line_rear.push(end);
            let mesh = Mesh::new_line(ctx, &line_rear, 2f32)?;

            graphics::set_color(ctx, Color::from_rgb(0, 191, 255))?;
            mesh.draw(ctx, Point2::new(0f32, 0f32), 0f32)?;
        }

        Ok(())
    }
}
