use ggez::*;
use ggez::graphics::*;
use super::*;

pub struct Ratio {
    pub gear: i32,
    pub ratio: f32,
    pub acceleration: GraphLine,
    pub max_speed: i32,
    pub differential: f32,
}

pub struct StupidGraphData {
    pub ratios: BTreeMap<i32, Ratio>,
    pub lateral_acceleration: GraphLine,
    pub max_rotations: f32,
    pub max_rotations_gear: i32,
    pub max_rotations_rpm: i32,
    pub max_ratio: f32,
}

impl StupidGraphData {
    pub fn new() -> StupidGraphData {
        StupidGraphData {
            ratios: BTreeMap::new(),
            lateral_acceleration: GraphLine::new(1, false, true),
            max_rotations: 1f32,
            max_rotations_gear: 1i32,
            max_rotations_rpm: 1i32,
            max_ratio: 1f32,
        }
    }

    pub fn draw(
        &self,
        ctx: &mut Context,
        power: &PowerGraphData,
        screen_size: &Point2,
        current_gear: i32,
        max_rpm: i32,
    ) -> GameResult<()> {
        if !self.ratios.is_empty() && power.torque.values.len() > 1 {
            let max_gear = self.ratios
                .keys()
                .max_by_key(|x| x.clone())
                .unwrap()
                .clone();

            let max_ratio = self.ratios
                .iter()
                .max_by(|x, y| {
                    if x.1.ratio <= y.1.ratio {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                })
                .map(|x| x.1.ratio)
                .unwrap_or(300f32);

            let y_scale = screen_size.y * 0.95 / (max_ratio * power.torque.max_value);

            let mut color = WHITE;

            for r in self.ratios.iter() {
                let (gear, ratio) = r;
                let alpha = *gear as f32 / max_gear as f32;

                let x_scale = screen_size.x
                    / (self.max_rotations * (max_rpm as f32 / self.max_rotations_rpm as f32)
                        * ratio.ratio);

                let mut points = vec![];
                for (r, t) in power.torque.values.iter() {
                    points.push(Point2::new(
                        *r as f32 * x_scale,
                        screen_size.y - y_scale * t * ratio.ratio,
                    ));
                }

                color.a = alpha;
                graphics::set_color(ctx, color)?;
                graphics::line(ctx, &points, 2f32)?;

                if ratio.gear == current_gear {
                    let dot = Point2::new(
                        x_scale * power.torque.current_value.0 as f32,
                        screen_size.y - y_scale * power.torque.current_value.1 * ratio.ratio,
                    );
                    graphics::set_color(ctx, Color::from_rgb(255, 140, 0))?;
                    graphics::circle(ctx, DrawMode::Fill, dot, 3f32, 1f32)?;
                }
            }
        }
        Ok(())
    }
}
