use super::nets::*;
use super::*;
use ggez::graphics::*;
use ggez::*;

pub struct Ratio {
    pub gear: i32,
    pub ratio: f32,
    pub differential: f32,
}

pub struct StupidGraphData {
    pub ratios: BTreeMap<i32, Ratio>,
    pub max_rotations: f32,
    pub max_rotations_rpm: f32,
    pub lateral_acceleration: GraphLine,
    pub longtitudal_acceleration: GraphLine,
    pub braking_acceleration: GraphLine,
    pub track_length: f32,
}

impl StupidGraphData {
    pub fn new(track_length: f32) -> StupidGraphData {
        StupidGraphData {
            ratios: BTreeMap::new(),
            lateral_acceleration: GraphLine::new(3, false, true, GraphRegion::Left, 3)
                .with_width(1f32),
            longtitudal_acceleration: GraphLine::new(3, false, true, GraphRegion::Left, 3)
                .with_width(1f32)
                .zero_on_current(true),
            braking_acceleration: GraphLine::new(3, false, true, GraphRegion::Left, 3)
                .with_width(1f32)
                .zero_on_current(true),
            max_rotations: 1f32,
            max_rotations_rpm: 0f32,
            track_length,
        }
    }

    pub fn update(
        &mut self,
        gear: i32,
        rpm: f32,
        diff_percent: f32,
        tyre_rps: f32,
        gear_ratio: f32,
        inputs: &Inputs,
    ) {
        if self.max_rotations < tyre_rps {
            self.max_rotations = tyre_rps;
            self.max_rotations_rpm = rpm;
        }

        let entry = self.ratios.entry(gear).or_insert(Ratio {
            gear,
            ratio: gear_ratio,
            differential: diff_percent,
        });

        if inputs.throttle > 0.2 && inputs.clutch == 0f32 && inputs.brake == 0f32 {
            entry.differential = diff_percent;
            entry.ratio = gear_ratio;
        }
    }

    pub fn add_ggv(
        &mut self,
        _gear: i32,
        position: f32,
        lateral: f32,
        longtitudal: f32,
        input: &Inputs,
        crash_state: u32,
    ) {
        let crash = crash_state != 0;
        let throttle = input.throttle > 0.01;
        let brake = input.brake > 0.01;

        if (lateral.abs() / 9.8) < 10f32 {
            self.lateral_acceleration
                .add(position as i32, lateral.abs() / 9.8, crash);
        };
        if position > self.track_length {
            self.track_length = position;
        }
        if (longtitudal.abs() / 9.8) < 10f32 {
            if longtitudal < 0f32 {
                self.longtitudal_acceleration.add(
                    position as i32,
                    -longtitudal / 9.8,
                    crash || !throttle,
                );
                self.braking_acceleration.add(position as i32, 0f32, false);
            } else if longtitudal > 0f32 {
                self.braking_acceleration
                    .add(position as i32, longtitudal / 9.8, crash || !brake);
                self.longtitudal_acceleration
                    .add(position as i32, 0f32, false);
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

            let ord_f32 = |x: &f32, y: &f32| -> Ordering {
                if x <= y {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            };

            let max_ratio = self.ratios
                .iter()
                .max_by(|x, y| ord_f32(&x.1.ratio, &y.1.ratio))
                .map(|x| x.1.ratio.clone())
                .unwrap();

            let min_ratio = self.ratios
                .iter()
                .min_by(|x, y| ord_f32(&x.1.ratio, &y.1.ratio))
                .map(|x| x.1.ratio.clone())
                .unwrap();

            let y_max = max_ratio * power.torque.max_value * 1.1;

            let mut color = WHITE;

            self.lateral_acceleration.draw(
                ctx,
                Color::from_rgb(128, 0, 255),
                Color::from_rgb(177, 100, 255),
                screen_size,
                &Point2::new(self.track_length, 10f32),
            )?;

            self.longtitudal_acceleration.draw(
                ctx,
                Color::from_rgb(34, 177, 76),
                Color::from_rgb(128, 255, 0),
                screen_size,
                &Point2::new(self.track_length, 10f32),
            )?;

            self.braking_acceleration.draw(
                ctx,
                Color::from_rgb(236, 87, 15),
                Color::from_rgb(250, 0, 0),
                screen_size,
                &Point2::new(self.track_length, 10f32),
            )?;

            for r in self.ratios.iter_mut() {
                let (gear, mut ratio) = r;
                let alpha = *gear as f32 / max_gear as f32;

                let x_max = max_rpm as f32 / min_ratio;

                let mut points = vec![];
                for (r, t) in power.torque.values.iter() {
                    points.push(scale_left(
                        (*r as f32 / ratio.ratio) / x_max,
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
                        (power.torque.current_value.0 as f32 / ratio.ratio) / x_max,
                        (power.torque.current_value.1 * ratio.ratio) / y_max,
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
