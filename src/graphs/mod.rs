pub mod nets;

mod rollndiff;
mod gears;

pub use self::rollndiff::*;
pub use self::gears::*;
use self::nets::*;

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
    region: GraphRegion,
}

pub enum GraphRegion {
    Left,
    TopRight,
    BottomRight,
}

impl GraphLine {
    pub fn new(step: i32, draw_dot: bool, draw_shadow: bool, region: GraphRegion) -> GraphLine {
        GraphLine {
            step,
            draw_shadow,
            draw_dot,
            values: BTreeMap::new(),
            shadow: VecDeque::new(),
            current_value: (0, 0f32),
            max_value: 1f32,
            cache: None,
            region,
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
                if self.shadow.len() > 50 || y < 0f32 {
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
        screen_size: &Point2,
        max_values: &Point2,
    ) -> GameResult<()> {
        if self.values.len() > 1 {
            if self.cache.is_none() {
                let v: Vec<Point2> = self.values
                    .iter()
                    .map(|(k, v)| {
                        let x = *k as f32 / max_values.x;
                        let y = v / max_values.y;
                        self.scale_point(x, y, screen_size)
                    })
                    .collect();

                let mesh = Mesh::new_line(ctx, &v, 2f32)?;
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

                let x = self.current_value.0 as f32 / max_values.x;
                let y = self.current_value.1 / max_values.y;
                let dot = self.scale_point(x, y, screen_size);
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
                        let x = dot.0 as f32 / max_values.x;
                        let y = dot.1 / max_values.y;

                        let point = self.scale_point(x, y, screen_size);

                        let x = last_dot.0 as f32 / max_values.x;
                        let y = last_dot.1 / max_values.y;
                        let last_point = self.scale_point(x, y, screen_size);

                        graphics::line(ctx, &[point, last_point], 1f32)?;

                        last_dot = *dot.clone();
                    }
                }
            }
        }
        Ok(())
    }

    pub fn scale_point(&self, x: f32, y: f32, screen_size: &Point2) -> Point2 {
        match self.region {
            GraphRegion::Left => scale_left(x, y, screen_size),
            GraphRegion::TopRight => scale_right_top(x, y, screen_size),
            GraphRegion::BottomRight => scale_right_bottom(x, y, screen_size),
        }
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
            throttle: GraphLine::new(rpm_step, false, false, GraphRegion::TopRight),
            torque: GraphLine::new(rpm_step, true, true, GraphRegion::TopRight),
            power: GraphLine::new(rpm_step, true, true, GraphRegion::TopRight),
        }
    }

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        screen_height: f32,
        screen_width: f32,
        max_rpm: i32,
        graph_height: f32,
    ) -> GameResult<()> {
        let throttle_color = Color::from_rgb(147, 197, 67);
        let torque_color = Color::from_rgb(67, 67, 197);
        let torque_dot = Color::from_rgb(255, 201, 14);
        let hp_color = Color::from_rgb(197, 67, 67);
        let hp_dot = Color::from_rgb(86, 226, 86);

        let power_max = Point2::new(max_rpm as f32, graph_height);
        let screen_size = Point2::new(screen_width, screen_height);

        self.throttle.draw(
            ctx,
            throttle_color,
            throttle_color,
            &screen_size,
            &Point2::new(max_rpm as f32, 1f32),
        )?;
        self.torque
            .draw(ctx, torque_color, torque_dot, &screen_size, &power_max)?;
        self.power
            .draw(ctx, hp_color, hp_dot, &screen_size, &power_max)?;

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
