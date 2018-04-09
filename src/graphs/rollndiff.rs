use ggez::*;
use ggez::graphics::*;
use definitions::*;

use std::time::Duration;
use std::collections::VecDeque;

pub struct RollGraphData {}

impl RollGraphData {
    pub fn draw(
        &self,
        ctx: &mut Context,
        sm: &SharedMemory,
        screen_size: &Point2,
    ) -> GameResult<()> {
        let center = Point2::new(screen_size.x * 0.8, screen_size.y * 0.35);

        let width = 100.0;
        let height = 50.0;

        let mult = 500f32;

        let (tyre_yf, tyre_yr) = linearize(&sm.mTyreY, width, height, mult);

        let mesh = MeshBuilder::new()
            .line(&tyre_yf, 2f32)
            .line(&tyre_yr, 2f32)
            .build(ctx)?;
        graphics::set_color(ctx, Color::from_rgb(200, 150, 120))?;
        mesh.draw(ctx, center, 0f32)?;

        let (tyre_stf, tyre_str) = linearize(&sm.mSuspensionTravel, width, height, -mult);

        let mesh = MeshBuilder::new()
            .line(&tyre_stf, 2f32)
            .line(&tyre_str, 2f32)
            .build(ctx)?;
        graphics::set_color(ctx, Color::from_rgb(255, 100, 0))?;
        mesh.draw(ctx, center, 0f32)?;

        Ok(())
    }
}

fn linearize(
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

pub struct DiffGraphData {
    current_time: Duration,
    current_diff: f32,
    start_time: Duration,
    start_diff: f32,
    mesh: VecDeque<(Duration, f32)>,
}

impl DiffGraphData {
    pub fn new() -> DiffGraphData {
        let dur = Duration::from_secs(0);
        DiffGraphData {
            current_time: dur,
            current_diff: 0f32,
            start_time: dur,
            start_diff: 0f32,
            mesh: VecDeque::new(),
        }
    }

    pub fn add(&mut self, diff: f32, time_dur: Duration) {
        let start_time = time_dur.checked_sub(Duration::from_secs(4));

        self.mesh.push_back((self.current_time, self.current_diff));

        self.current_diff = diff;
        self.current_time = time_dur;

        if let Some(start_time) = start_time {
            let mut start: Option<(Duration, f32)> = None;

            for _i in 0..self.mesh.len() {
                if self.mesh[0].0 < start_time {
                    start = self.mesh.pop_front();
                } else {
                    break;
                }
            }

            if let Some((dur, diff)) = start {
                self.start_time = dur;
                self.start_diff = diff;
            } else {
                self.start_time = start_time;
                self.start_diff = 0f32;
            }
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, screen_size: &Point2) -> GameResult<()> {
        let end = self.current_time.clone();

        let x_start = screen_size.x * 0.6;
        let x_end = screen_size.x - (screen_size.x - x_start) * 0.2;
        let y_start = screen_size.y * 0.525;
        let y_end = screen_size.y;

        let y_coefficient = |diff: &f32| -> f32 { y_start + ((y_end - y_start) * diff) };

        let x_coefficient = |dur: &Duration| -> f32 {
            let to_end = end - *dur;
            let coeff =
                (to_end.as_secs() as f32 * 1000f32 + to_end.subsec_millis() as f32) / 4000f32;

            x_end - (x_end - x_start) * coeff
        };

        if self.mesh.len() > 1 {
            // let start = Point2::new(x_start, y_coefficient(&self.start_diff));
            let end = Point2::new(
                x_coefficient(&self.current_time),
                y_coefficient(&self.current_diff),
            );

            let mut line: Vec<Point2> = self.mesh
                .iter()
                .map(|tup| Point2::new(x_coefficient(&tup.0), y_coefficient(&tup.1)))
                .collect();

            line.push(end);

            let mesh = Mesh::new_line(ctx, &line, 2f32)?;

            graphics::set_color(ctx, Color::from_rgb(97, 97, 200))?;
            mesh.draw(ctx, Point2::new(0f32, 0f32), 0f32)?;
        }
        Ok(())
    }
}
