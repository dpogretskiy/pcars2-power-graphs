use definitions::*;
use std::collections::BTreeMap;
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
    max_power: f32,
    graph_height: f32,
    current_power: f32,
    screen_width: u32,
    screen_height: u32,
    current_dots: [Point2; 2],
}

type ThrottleInput = f32;
type RPM = i32;
type Torque = f32;
type BoostPressure = f32;

pub struct PowerGraphData {
    pub data: BTreeMap<RPM, (ThrottleInput, Torque, BoostPressure)>,
}

pub struct ShiftGraphData {
    pub data: BTreeMap<i32, Vec<(RPM, RPM)>>,
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
        screen_width: u32,
        screen_height: u32,
    ) -> PC2App {
        PC2App {
            shared_data: sm,
            local_copy: unsafe { std::ptr::read_volatile(sm) },
            assets: Assets::load(ctx),
            current_gear: 0,
            current_rpm: 0,
            max_rpm: 1,
            max_power: 1f32,
            current_power: 1f32,
            power_data: PowerGraphData {
                data: BTreeMap::new(),
            },
            shift_data: ShiftGraphData {
                data: BTreeMap::new(),
            },
            current_car: String::new(),
            current_track: String::new(),
            screen_width,
            screen_height,
            graph_height: 300f32,
            current_dots: [Point2::new(-10f32, -10f32), Point2::new(-10f32, -10f32)],
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
            self.power_data.data = BTreeMap::new();
            self.shift_data.data = BTreeMap::new();
            self.max_power = 1f32;
            self.current_power = 1f32;
            self.graph_height = 300f32;
            self.current_dots = [Point2::new(-10f32, -10f32), Point2::new(-10f32, -10f32)];
            // let mut title = car_name;
            // title.push_str(" : ");
            // title.push_str(&track_name);

            // ctx.conf.window_setup.title = title;
        }

        let current_rpm = local_copy.mRpm as i32;
        let rpm = current_rpm - current_rpm % 10;
        self.current_rpm = rpm;
        let current_torque = local_copy.mEngineTorque;
        let throttle = local_copy.mThrottle;
        let boost_pressure = local_copy.mTurboBoostPressure;
        let power = (current_torque * rpm as f32 / 9548.8) / 0.7457;
        self.current_dots = {
            let y_coeff = self.screen_height as f32 / (self.graph_height * 1.1);
            [
                Point2::new(
                    rpm as f32 * (self.screen_width as f32 / self.max_rpm as f32),
                    self.screen_height as f32 - current_torque * y_coeff,
                ),
                Point2::new(
                    rpm as f32 * (self.screen_width as f32 / self.max_rpm as f32),
                    self.screen_height as f32 - power * y_coeff,
                ),
            ]
        };

        if throttle > 0.9999 && local_copy.mClutch < 0.0001 {
            let data = self.power_data.data.entry(rpm).or_insert((
                throttle,
                current_torque,
                boost_pressure,
            ));

            self.max_power = self.max_power.max(power);
            self.graph_height = self.graph_height.max(current_torque).max(power);
            self.current_power = power;

            if data.1 < current_torque {
                data.0 = throttle;
                data.1 = current_torque;
                data.2 = boost_pressure;
            }
        }

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

        let mut throttle_line = Vec::new();
        let mut torque_line = Vec::new();
        let mut hp_line = Vec::new();

        for (rpm, ref triple) in self.power_data.data.iter() {
            let (th, tq, _bp) = *triple.clone();

            let x = *rpm as f32 * (self.screen_width as f32 / self.max_rpm as f32);
            let y_coeff = self.screen_height as f32 / (self.graph_height * 1.1);

            let power = (tq as f32 * *rpm as f32 / 9548.8) / 0.7457;
            throttle_line.push(Point2::new(
                x,
                self.screen_height as f32 - (th * self.screen_height as f32 * 0.95),
            ));
            torque_line.push(Point2::new(x, self.screen_height as f32 - tq * y_coeff));
            hp_line.push(Point2::new(x, self.screen_height as f32 - power * y_coeff));
        }
        //net
        {
            graphics::set_color(ctx, Color::from_rgba(127, 127, 127, 127))?;
            for rpm_vert in (0..self.max_rpm as u32).step_by(1000) {
                let x = rpm_vert as f32 * (self.screen_width as f32 / self.max_rpm as f32);
                graphics::line(
                    ctx,
                    &[
                        Point2::new(x, self.screen_height as f32 * 0.05),
                        Point2::new(x, self.screen_height as f32),
                    ],
                    1f32,
                )?;
            }
            for horizontal in (0..(self.graph_height * 1.1) as u32).step_by(200) {
                let y = self.screen_height as f32
                    - (horizontal as f32 * self.screen_height as f32 / self.graph_height);
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

        if throttle_line.len() > 1 {
            graphics::set_color(ctx, Color::from_rgb(147, 197, 67))?;
            graphics::line(ctx, &throttle_line, 2f32)?;
        }
        if torque_line.len() > 1 {
            graphics::set_color(ctx, Color::from_rgb(67, 67, 197))?;
            graphics::line(ctx, &torque_line, 2f32)?;
        }
        if hp_line.len() > 1 {
            graphics::set_color(ctx, Color::from_rgb(197, 67, 67))?;
            graphics::line(ctx, &hp_line, 2f32)?;
        }

        //power_dots =
        graphics::set_color(ctx, Color::from_rgb(255, 201, 11))?;
        for dot in self.current_dots.iter() {
            graphics::circle(ctx, DrawMode::Fill, dot.clone(), 4f32, 1f32)?;
        }

        //text;
        let target = graphics::Point2::new(2f32, 2f32);
        let ix = graphics::Text::new(
            ctx,
            &format!(
                "MAXHP: {} MAXRPM: {}, GEAR: {}, RPM: {}, HP: {}",
                self.max_power,
                self.max_rpm,
                self.current_gear,
                self.current_rpm,
                self.current_power,
            ),
            &self.assets.font,
        )?;
        graphics::set_color(ctx, WHITE)?;
        graphics::draw(ctx, &ix, target, 0f32)?;

        graphics::present(ctx);

        if timer::get_ticks(ctx) % 100 == 0 {
            // if timer::get_ticks(ctx) % 1000 == 0 {
            //     println!("Power data: {:?}", self.power_data.data);
            //     println!("Shift data: {:?}", self.shift_data.data);
            // }
            println!("FPS: {}", timer::get_fps(ctx));
            println!("Seq frame: {}", self.local_copy.mSequenceNumber);
        }
        Ok(())
    }
}
