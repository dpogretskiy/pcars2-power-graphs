pub mod nets;

mod gears;
mod rollndiff;

pub use self::gears::*;
use self::nets::*;
pub use self::rollndiff::*;

use definitions::*;
use ggez::graphics::*;
use ggez::*;
use std::cmp::Ordering;
use std::collections::{BTreeMap, VecDeque};
use std::f32;

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
    smoothening: usize,
    line_width: f32,
    zoc: bool,
}

pub enum GraphRegion {
    Left,
    TopRight,
    BottomRight,
}

impl GraphLine {
    pub fn new(
        step: i32,
        draw_dot: bool,
        draw_shadow: bool,
        region: GraphRegion,
        smoothening: usize,
    ) -> GraphLine {
        let smoothening = smoothening.max(1);

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
            smoothening,
            line_width: 2f32,
            zoc: false,
        }
    }

    pub fn with_width(self, width: f32) -> Self {
        let mut s = self;
        s.line_width = width;
        s
    }

    pub fn zero_on_current(self, zoc: bool) -> Self {
        let mut s = self;
        s.zoc = zoc;
        s
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
            } else if self.zoc {
                self.cache = None;
                let step_x = x - x % self.step;
                let entry = self.values.entry(step_x).or_insert(0f32);
                *entry = entry.max(0f32);
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
        if self.values.len() > self.smoothening {
            if self.cache.is_none() {
                let values = self.values.iter().collect::<Vec<_>>();

                let smooth = values
                    .windows(self.smoothening)
                    .map(|v| {
                        let point = v[0];
                        let avg: f32 = v.iter().map(|p| *p.1).sum::<f32>() / v.len() as f32;

                        let x = *point.0 as f32 / max_values.x;
                        let y = avg / max_values.y;

                        self.scale_point(x, y, screen_size)
                    })
                    .collect::<Vec<_>>();

                let mesh = Mesh::new_line(ctx, &smooth, self.line_width)?;
                self.cache = Some(mesh);
                // .map(|iter| iter.average());

                // let src: Vec<_> = self.values.iter().collect();
                // let len = src.len();

                // let mut vec: Vec<Point2> = Vec::with_capacity(len);

                // for i in 0..len {
                //     let (k, _) = src[i];
                //     let mut start = if i - severity > 0 { i - severity } else { len };
                //     let mut end = (i + severity).min(len);

                //     let mut sum = 0f32;

                //     for j in start..end {
                //         sum += src[j].1;
                //     }

                //     let avg = sum / (end - start) as f32;
                //     let x = *k as f32 / max_values.x;
                //     let y = avg / max_values.y;
                //     if x > 0f32 && x < 1f32 && y >= 0f32 && y < 1f32 {
                //         vec.push(self.scale_point(x, y, screen_size));
                //     }
                // }

                // if vec.len() > 1 {
                //     let mesh = Mesh::new_line(ctx, &vec, self.line_width)?;
                //     self.cache = Some(mesh)
                // }
            }
        }

        self.draw_old(ctx, line_color, dot_color, screen_size, max_values)?;
        Ok(())
    }

    pub fn draw_old(
        &mut self,
        ctx: &mut Context,
        line_color: Color,
        dot_color: Color,
        screen_size: &Point2,
        max_values: &Point2,
    ) -> GameResult<()> {
        if self.values.len() > 1 {
            if let Some(ref cache) = self.cache {
                cache.draw_ex(
                    ctx,
                    DrawParam {
                        color: Some(line_color),
                        ..Default::default()
                    },
                )?;
            }

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
            throttle: GraphLine::new(rpm_step, false, false, GraphRegion::TopRight, 1)
                .zero_on_current(false),
            torque: GraphLine::new(rpm_step, true, true, GraphRegion::TopRight, 3)
                .zero_on_current(false),
            power: GraphLine::new(rpm_step, true, true, GraphRegion::TopRight, 3)
                .zero_on_current(false),
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
