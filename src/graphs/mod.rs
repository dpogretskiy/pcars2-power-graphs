pub mod nets;

mod rollndiff;

pub use self::rollndiff::*;

use definitions::*;
use std::collections::{BTreeMap, VecDeque};
use ggez::*;
use ggez::graphics::*;
use std::f32;
use std::cmp::Ordering;

pub struct GraphLine {
    step: i32,
    draw_dot: bool,
    draw_shadow: bool,
    values: BTreeMap<i32, f32>,
    shadow: VecDeque<(i32, f32)>,
    pub current_value: (i32, f32),
    pub max_value: f32,
    pub cache: Option<Mesh>,
}

impl GraphLine {
    pub fn new(step: i32, draw_dot: bool, draw_shadow: bool) -> GraphLine {
        GraphLine {
            step,
            draw_shadow,
            draw_dot,
            values: BTreeMap::new(),
            shadow: VecDeque::new(),
            current_value: (0, 0f32),
            max_value: 1f32,
            cache: None,
        }
    }

    pub fn add(&mut self, x: i32, y: f32, current_only: bool) {
        if self.current_value != (x, y) {
            self.current_value = (x, y);
            self.max_value = self.max_value.max(y);

            if self.draw_shadow {
                if y >= 0f32 {
                    self.shadow.push_back((x, y));
                }
                if self.shadow.len() > 100 || y < 0f32 {
                    self.shadow.pop_front();
                }
            }
            if !current_only {
                self.cache = None;
                let step_x = x - x % self.step;
                let entry = self.values.entry(step_x).or_insert(y);
                *entry = entry.max(y);
            }
        }
    }

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        line_color: Color,
        dot_color: Color,
        screen_height: f32,
        scale: Point2,
    ) -> GameResult<()> {
        if self.values.len() > 1 {
            if self.cache.is_none() {
                let v: Vec<_> = self.values
                    .iter()
                    .map(|(k, v)| scale_mult(Point2::new(*k as f32, *v), &scale, screen_height))
                    .collect();

                let mesh = MeshBuilder::new().line(&v, 2f32).build(ctx)?;
                self.cache = Some(mesh);
            }

            // graphics::set_color(ctx, line_color)?;
            if let Some(ref cache) = self.cache {
                cache.draw_ex(
                    ctx,
                    DrawParam {
                        color: Some(line_color),
                        ..Default::default()
                    },
                )?;
            }

            // graphics::line(ctx, &v, 2f32)?;
            if self.draw_dot && !self.draw_shadow && self.current_value.1 > 0f32 {
                graphics::set_color(ctx, dot_color)?;
                let dot = scale_mult(
                    Point2::new(self.current_value.0 as f32, self.current_value.1),
                    &scale,
                    screen_height,
                );
                graphics::circle(ctx, DrawMode::Fill, dot, 3f32, 1f32)?;
            }

            if self.draw_shadow && !self.shadow.is_empty() {
                let alpha_step = 1f32 / self.shadow.len() as f32;
                let mut shadow_color = dot_color.clone();
                shadow_color.a = 1f32;
                let mut last_dot = self.shadow.back().unwrap().clone();

                for ref dot in self.shadow.iter().rev() {
                    shadow_color.a -= alpha_step;
                    if dot.1 > 0f32 || last_dot.1 > 0f32 {
                        graphics::set_color(ctx, shadow_color.clone())?;
                        let point =
                            scale_mult(Point2::new(dot.0 as f32, dot.1), &scale, screen_height);
                        let last_point = scale_mult(
                            Point2::new(last_dot.0 as f32, last_dot.1),
                            &scale,
                            screen_height,
                        );

                        graphics::line(ctx, &[point, last_point], 2f32)?;

                        last_dot = *dot.clone();
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct PowerGraphData {
    pub throttle: GraphLine,
    pub torque: GraphLine,
    pub power: GraphLine,
}

impl PowerGraphData {
    pub fn new(rpm_step: i32) -> PowerGraphData {
        PowerGraphData {
            throttle: GraphLine::new(rpm_step, false, false),
            torque: GraphLine::new(rpm_step, true, true),
            power: GraphLine::new(rpm_step, true, true),
        }
    }

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        screen_height: f32,
        screen_width: f32,
        max_rpm: i32,
    ) -> GameResult<()> {
        //net
        let graph_height = self.power.max_value.max(self.torque.max_value) * 1.2;

        let throttle_color = Color::from_rgb(147, 197, 67);
        let torque_color = Color::from_rgb(67, 67, 197);
        let torque_dot = Color::from_rgb(255, 201, 14);
        let hp_color = Color::from_rgb(197, 67, 67);
        let hp_dot = Color::from_rgb(86, 226, 86);

        let x_scale = screen_width / max_rpm as f32;
        let y_scale = screen_height / graph_height;
        let power_scale = Point2::new(x_scale, y_scale);

        self.throttle.draw(
            ctx,
            throttle_color,
            throttle_color,
            screen_height,
            Point2::new(x_scale, screen_height * 0.95),
        )?;
        self.torque.draw(
            ctx,
            torque_color,
            torque_dot,
            screen_height,
            power_scale.clone(),
        )?;
        self.power
            .draw(ctx, hp_color, hp_dot, screen_height, power_scale.clone())?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Inputs {
    pub throttle: f32,
    pub brake: f32,
    pub clutch: f32,
    pub steering: f32,
}

impl Inputs {
    pub fn from(sm: &SharedMemory) -> Inputs {
        Inputs {
            throttle: sm.mThrottle,
            brake: sm.mBrake,
            clutch: sm.mClutch,
            steering: sm.mSteering,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ratio {
    pub gear: i32,
    pub ratio: f32,
    pub acceleration: f32,
    pub velocity: f32,
    pub x_velocity: f32,
    pub rpm: f32,
    pub differential: f32,
    pub inputs: Inputs,
}

#[derive(Debug)]
pub struct StupidGraphData {
    pub ratios: BTreeMap<i32, Ratio>,
    pub max_rotations: f32,
    pub max_rotations_gear: i32,
    pub max_rotations_rpm: i32,
    pub max_ratio: f32,
}

impl StupidGraphData {
    pub fn new() -> StupidGraphData {
        StupidGraphData {
            ratios: BTreeMap::new(),
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

fn scale_mult(x: Point2, y: &Point2, screen_height: f32) -> Point2 {
    Point2::new(x.x * y.x, screen_height - x.y * y.y)
}
