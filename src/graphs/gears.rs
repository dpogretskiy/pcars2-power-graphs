use super::nets::*;
use super::*;
use ggez::graphics::*;
use ggez::*;

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
            lateral_acceleration: GraphLine::new(1, false, true, GraphRegion::Left),
            max_rotations: 1f32,
            max_rotations_gear: 1,
            max_rotations_rpm: 1,
            max_ratio: 1f32,
        }
    }

    pub fn add_ggv(&mut self, gear: i32, speed: f32, lateral: f32, longtitudal: f32) {
        self.lateral_acceleration
            .add(speed as i32, lateral.abs() / 9.8, false);
        let mut geared = self.ratios.get_mut(&gear);
        if let Some(ref mut g) = geared {
            if longtitudal < 0 {
                g.acceleration.add(speed as i32, -longtitudal / 9.8, false);
            }
        }
    }

    pub fn draw(
        &mut self,
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
                .unwrap_or(10f32);

            let y_max = max_ratio * power.torque.max_value * 1.1;

            let mut color = WHITE;

            let max_speed = 300f32;

            self.lateral_acceleration.draw(
                ctx,
                Color::from_rgb(50, 150, 50),
                Color::from_rgb(50, 50, 150),
                screen_size,
                &Point2::new(max_speed, 40f32),
            )?;

            for r in self.ratios.iter_mut() {
                let (gear, mut ratio) = r;
                let alpha = *gear as f32 / max_gear as f32;

                let x_max = (self.max_rotations * (max_rpm as f32 / self.max_rotations_rpm as f32))
                    / ratio.ratio;

                ratio.acceleration.draw(
                    ctx,
                    Color::from_rgb(150, 0, 0),
                    Color::from_rgb(150, 150, 0),
                    screen_size,
                    &Point2::new(max_speed, 40f32),
                )?;

                let mut points = vec![];
                for (r, t) in power.torque.values.iter() {
                    points.push(scale_left(
                        *r as f32 / x_max,
                        (t * ratio.ratio) / y_max,
                        screen_size,
                    ));
                }

                color.a = alpha;
                graphics::set_color(ctx, color)?;
                graphics::line(ctx, &points, 2f32)?;

                if ratio.gear == current_gear {
                    // let dot = Point2::new(
                    //     x_scale * power.torque.current_value.0 as f32,
                    //     screen_size.y - y_scale * power.torque.current_value.1 * ratio.ratio,
                    // );
                    let dot = scale_left(
                        power.torque.current_value.0 as f32 / x_scale,
                        (power.torque.current_value.1 * ratio.ratio) / y_scale,
                        screen_size,
                    );
                    graphics::set_color(ctx, Color::from_rgb(255, 140, 0))?;
                    graphics::circle(ctx, DrawMode::Fill, dot, 3f32, 1f32)?;
                }
            }
        }
        Ok(())
    }
}
