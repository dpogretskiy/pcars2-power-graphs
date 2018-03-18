use definitions::*;
use std::collections::{BTreeMap, VecDeque};
use std;
use ggez::*;
use ggez::graphics::*;

pub struct PC2App {
    shared_data: *const SharedMemory,
    local_copy: SharedMemory,
    assets: Assets,
    current_gear: i32,
    current_rpm: i32,
    max_rpm: i32,
    current_car: String,
    current_track: String,
    power_data: PowerGraphData,
    shift_data: ShiftGraphData,
    optimized_text: OptimizedText,
    screen_width: f32,
    screen_height: f32,
}

pub struct PowerGraphData {
    pub throttle: GraphLine,
    pub torque: GraphLine,
    pub power: GraphLine,
}

impl PowerGraphData {
    pub fn new() -> PowerGraphData {
        PowerGraphData {
            throttle: GraphLine::new(10, false, false),
            torque: GraphLine::new(10, true, true),
            power: GraphLine::new(10, true, true),
        }
    }
}

pub struct ShiftGraphData {
    pub data: BTreeMap<i32, Vec<(i32, i32)>>,
}

pub struct Assets {
    font: graphics::Font,
}

impl Assets {
    pub fn load(ctx: &mut Context) -> Assets {
        Assets {
            font: graphics::Font::new(ctx, "/DejaVuSerif.ttf", 18).unwrap(),
        }
    }
}

impl PC2App {
    pub fn new(
        ctx: &mut Context,
        sm: *const SharedMemory,
        screen_width: f32,
        screen_height: f32,
    ) -> PC2App {
        let assets = Assets::load(ctx);
        let fragments = vec![
            graphics::Text::new(ctx, "MAXHP: ", &assets.font).unwrap(),
            graphics::Text::new(ctx, "MAXRPM: ", &assets.font).unwrap(),
            graphics::Text::new(ctx, "GEAR: ", &assets.font).unwrap(),
            graphics::Text::new(ctx, "RPM: ", &assets.font).unwrap(),
            graphics::Text::new(ctx, "HP: ", &assets.font).unwrap(),
        ];

        let optimized_text = OptimizedText::new(fragments);

        // "MAXHP: {} MAXRPM: {}, GEAR: {}, RPM: {}, HP: {}",
        PC2App {
            shared_data: sm,
            local_copy: unsafe { std::ptr::read_volatile(sm) },
            assets,
            optimized_text,
            current_gear: 0,
            current_rpm: 0,
            max_rpm: 1,
            power_data: PowerGraphData::new(),
            shift_data: ShiftGraphData {
                data: BTreeMap::new(),
            },
            current_car: String::new(),
            current_track: String::new(),
            screen_width,
            screen_height,
        }
    }
}

impl event::EventHandler for PC2App {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let local_copy = unsafe { std::ptr::read_volatile(self.shared_data) };
        let update_index = local_copy.mSequenceNumber;

        if update_index % 2 != 0 || update_index == self.local_copy.mSequenceNumber {
            return Ok(());
        }

        let track_name = local_copy.mTrackLocation.clone().to_string();
        let car_name = local_copy.mCarName.clone().to_string();

        if track_name.is_empty() || car_name.is_empty() {
            return Ok(());
        }

        if self.current_car != car_name || self.current_track != track_name {
            self.current_car = car_name.clone();
            self.current_track = track_name.clone();
            self.max_rpm = local_copy.mMaxRPM as i32;
            self.power_data = PowerGraphData::new();
            self.shift_data.data = BTreeMap::new();
            // let mut title = car_name;
            // title.push_str(" : ");
            // title.push_str(&track_name);

            // ctx.conf.window_setup.title = title;
        }

        let current_rpm = local_copy.mRpm as i32;
        let rpm = current_rpm - current_rpm % 10;
        self.current_rpm = rpm;
        let throttle = local_copy.mThrottle;
        let torque = local_copy.mEngineTorque;
        let power = (torque * rpm as f32 / 9548.8) / 0.7457;

        let currents_only = !(throttle > 0.9999 && local_copy.mClutch < 0.0001);

        self.power_data.throttle.add(rpm, throttle, currents_only);
        self.power_data.torque.add(rpm, torque, currents_only);
        self.power_data.power.add(rpm, power, currents_only);

        if self.current_gear != local_copy.mGear {
            let old_gear = self.current_gear;
            let new_gear = local_copy.mGear;
            let old_rpm = self.current_rpm;
            let new_rpm = current_rpm;
            self.current_gear = new_gear;
            self.current_rpm = new_rpm;

            match new_gear {
                3 | 4 | 5 | 6 | 7 => {
                    if old_gear + 1 == new_gear {
                        let gn = old_gear * 10 + new_gear;
                        let v = self.shift_data.data.entry(gn).or_insert(Vec::new());
                        v.push((new_rpm, old_rpm));
                        if v.len() > 5 {
                            v.remove(0);
                        }
                    }
                }
                _ => {}
            }
        }

        self.local_copy = local_copy;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, Color::from_rgb(18, 31, 52))?;
        graphics::clear(ctx);

        // let graph_height = 1000f32;
        let graph_height = self.power_data
            .power
            .max_value
            .max(self.power_data.torque.max_value) * 1.2;

        let throttle_color = Color::from_rgb(147, 197, 67);
        let torque_color = Color::from_rgb(67, 67, 197);
        let torque_dot = Color::from_rgb(255, 201, 14);
        let hp_color = Color::from_rgb(197, 67, 67);
        let hp_dot = Color::from_rgb(86, 226, 86);

        let x_scale = self.screen_width / self.max_rpm as f32;
        let y_scale = self.screen_height / graph_height;

        self.power_data
            .throttle
            .set_scale(x_scale, self.screen_height * 0.95);
        self.power_data.torque.set_scale(x_scale, y_scale);
        self.power_data.power.set_scale(x_scale, y_scale);

        self.power_data
            .throttle
            .draw(ctx, throttle_color, throttle_color, self.screen_height)?;
        self.power_data
            .torque
            .draw(ctx, torque_color, torque_dot, self.screen_height)?;
        self.power_data
            .power
            .draw(ctx, hp_color, hp_dot, self.screen_height)?;

        //net
        {
            graphics::set_color(ctx, Color::from_rgba(127, 127, 127, 127))?;
            for rpm_vert in (0..self.max_rpm as u32).step_by(1000) {
                let x = rpm_vert as f32 * (self.screen_width / self.max_rpm as f32);
                graphics::line(
                    ctx,
                    &[
                        Point2::new(x, self.screen_height as f32 * 0.05),
                        Point2::new(x, self.screen_height as f32),
                    ],
                    1f32,
                )?;
            }
            for horizontal in (0..(graph_height * 1.2) as u32).step_by(200) {
                let y = self.screen_height
                    - (horizontal as f32 * self.screen_height as f32 / graph_height);
                graphics::line(
                    ctx,
                    &[
                        Point2::new(0f32, y),
                        Point2::new(self.screen_width as f32, y),
                    ],
                    1f32,
                )?;
            }
        }

        //text;
        let values = vec![
            self.power_data.power.max_value as i32,
            self.max_rpm,
            self.current_gear,
            self.current_rpm,
            self.power_data.power.current_value.1 as i32,
        ];

        self.optimized_text.draw(ctx, &self.assets.font, values)?;

        graphics::present(ctx);

        timer::yield_now();
        if timer::get_ticks(ctx) % 1000 == 0 {
            println!("FPS: {}", timer::get_fps(ctx));
        }
        Ok(())
    }
}

pub struct OptimizedText {
    names: Vec<graphics::Text>,
}

impl OptimizedText {
    pub fn new(names: Vec<graphics::Text>) -> OptimizedText {
        OptimizedText { names }
    }

    pub fn draw<T: ToString>(
        &self,
        ctx: &mut Context,
        font: &graphics::Font,
        values: Vec<T>,
    ) -> GameResult<()> {
        assert!(self.names.len() == values.len());
        let mut target = graphics::Point2::new(2f32, 2f32);

        graphics::set_color(ctx, WHITE)?;
        for tuple in self.names.iter().zip(values.iter()) {
            let (n, v) = tuple;
            graphics::draw(ctx, n, target, 0f32)?;
            target.x += n.width() as f32;
            let v_text = graphics::Text::new(ctx, &v.to_string(), &font)?;
            graphics::draw(ctx, &v_text, target, 0f32)?;
            target.x += v_text.width() as f32 + 5f32;
        }
        Ok(())
    }
}

pub struct GraphLine {
    step: i32,
    draw_dot: bool,
    draw_shadow: bool,
    values: BTreeMap<i32, f32>,
    shadow: VecDeque<(i32, f32)>,
    current_value: (i32, f32),
    scale: Point2,
    pub max_value: f32,
}

impl GraphLine {
    pub fn new(step: i32, draw_dot: bool, draw_shadow: bool) -> GraphLine {
        GraphLine {
            values: BTreeMap::new(),
            shadow: VecDeque::new(),
            max_value: 1f32,
            current_value: (0, 0f32),
            step,
            draw_dot,
            draw_shadow,
            scale: Point2::new(1f32, 1f32),
        }
    }

    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.scale = Point2::new(x, y);
    }

    pub fn add(&mut self, x: i32, y: f32, current_only: bool) {
        if self.current_value != (x, y) {
            self.current_value = (x, y);
            self.max_value = self.max_value.max(y);

            if self.draw_shadow && y > 0f32 {
                self.shadow.push_back((x, y));
                if self.shadow.len() > 100 {
                    self.shadow.pop_front();
                }
            }
            if !current_only {
                let step_x = x - x % self.step;
                let entry = self.values.entry(step_x).or_insert(y);
                *entry = entry.max(y);
            }
        }
    }

    pub fn draw(
        &self,
        ctx: &mut Context,
        line_color: Color,
        dot_color: Color,
        screen_height: f32,
    ) -> GameResult<()> {
        if self.values.len() > 1 {
            let v: Vec<_> = self.values
                .iter()
                .map(|(k, v)| scale_mult(Point2::new(*k as f32, *v), &self.scale, screen_height))
                .collect();
            graphics::set_color(ctx, line_color)?;
            graphics::line(ctx, &v, 2f32)?;
            if self.draw_dot && !self.draw_shadow && self.current_value.1 > 0f32 {
                graphics::set_color(ctx, dot_color)?;
                let dot = Point2::new(self.current_value.0 as f32, self.current_value.1);
                graphics::circle(
                    ctx,
                    DrawMode::Fill,
                    scale_mult(dot, &self.scale, screen_height),
                    3f32,
                    1f32,
                )?;
            }
            if self.draw_shadow {
                let alpha_step = 1f32 / self.shadow.len() as f32;
                let mut shadow_color = dot_color.clone();
                shadow_color.a = 1f32;
                let mut last_dot = self.current_value;
                for ref dot in self.shadow.iter().rev() {
                    shadow_color.a -= alpha_step;
                    if dot.1 > 0f32 || last_dot.1 > 0f32 {
                        graphics::set_color(ctx, shadow_color.clone())?;
                        let point = scale_mult(
                            Point2::new(dot.0 as f32, dot.1),
                            &self.scale,
                            screen_height,
                        );
                        let last_point = scale_mult(
                            Point2::new(last_dot.0 as f32, last_dot.1),
                            &self.scale,
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

fn scale_mult(x: Point2, y: &Point2, screen_height: f32) -> Point2 {
    Point2::new(x.x * y.x, screen_height - x.y * y.y)
}
