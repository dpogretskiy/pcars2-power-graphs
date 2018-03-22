use definitions::*;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std;
use ggez::*;
use ggez::graphics::*;
use std::f32;
use super::graphs::*;

pub enum Options {
    ShiftGraph,
}

pub struct PC2App {
    options: Vec<Options>,
    shared_data: *const SharedMemory,
    local_copy: SharedMemory,
    current_gear: i32,
    current_rpm: i32,
    max_rpm: i32,
    current_car: String,
    current_track: String,
    power_data: PowerGraphData,
    optimized_text: OptimizedText,
    numeric_text_cache: NumericTextCache,
    screen_width: f32,
    screen_height: f32,
    stupid_graphs: StupidGraphData,
}

pub struct NumericTextCache {
    numbers: HashMap<i32, graphics::Text>,
}

impl NumericTextCache {
    pub fn new(ctx: &mut Context, font: &graphics::Font) -> NumericTextCache {
        let mut numbers = HashMap::new();

        for number in (-1500..1501).chain((1510..20001).step_by(10)) {
            let txt = graphics::Text::new(ctx, &number.to_string(), font).unwrap();
            numbers.insert(number, txt);
        }

        NumericTextCache { numbers }
    }
}

impl PC2App {
    pub fn new(
        ctx: &mut Context,
        sm: *const SharedMemory,
        screen_width: f32,
        screen_height: f32,
    ) -> PC2App {
        let font = PC2App::load_font(ctx);
        let fragments = vec![
            graphics::Text::new(ctx, "MAXHP: ", &font).unwrap(),
            graphics::Text::new(ctx, "MAXRPM: ", &font).unwrap(),
            graphics::Text::new(ctx, "GEAR: ", &font).unwrap(),
            graphics::Text::new(ctx, "RPM: ", &font).unwrap(),
            graphics::Text::new(ctx, "HP: ", &font).unwrap(),
            graphics::Text::new(ctx, "R: ", &font).unwrap(),
            graphics::Text::new(ctx, "Rot: ", &font).unwrap(),
        ];
        let numeric_text_cache = NumericTextCache::new(ctx, &font);
        let optimized_text = OptimizedText::new(fragments);

        // "MAXHP: {} MAXRPM: {}, GEAR: {}, RPM: {}, HP: {}",
        PC2App {
            shared_data: sm,
            local_copy: unsafe { std::ptr::read_volatile(sm) },
            optimized_text,
            current_gear: 0,
            current_rpm: 0,
            max_rpm: 1,
            power_data: PowerGraphData::new(),
            current_car: String::new(),
            current_track: String::new(),
            screen_width,
            screen_height,
            numeric_text_cache,
            options: vec![],
            stupid_graphs: StupidGraphData::new(),
        }
    }

    pub fn load_font(ctx: &mut Context) -> graphics::Font {
        graphics::Font::new(ctx, "/DejaVuSerif.ttf", 18).unwrap()
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
            self.stupid_graphs = StupidGraphData::new();

            let mut title = car_name;
            title.push_str(" @ ");
            title.push_str(&track_name);
            graphics::get_window_mut(_ctx).set_title(&title).unwrap();
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

        self.current_gear = local_copy.mGear;

        //stupid stuff
        {
            if self.current_gear > 0 {
                let velocity = -local_copy.mLocalVelocity.z;
                let tyre_rps_arr = local_copy.mTyreRPS.clone();
                let differential = (tyre_rps_arr.data[Tyre::TyreRearLeft as usize]
                    - tyre_rps_arr.data[Tyre::TyreRearRight as usize])
                    .abs();
                let inputs = Inputs::from(&local_copy);
                let x_velocity = local_copy.mLocalVelocity.x;

                let tyre_rps = ((tyre_rps_arr.data[Tyre::TyreRearLeft as usize]
                    + tyre_rps_arr.data[Tyre::TyreRearRight as usize])
                    / 2f32)
                    .abs();

                let ratio = (rpm as f32 / tyre_rps);

                let acceleration = -local_copy.mLocalAcceleration.z;

                if self.stupid_graphs.max_ratio < ratio {
                    self.stupid_graphs.max_ratio = ratio;
                }

                if self.stupid_graphs.max_rotations < tyre_rps {
                    self.stupid_graphs.max_rotations = tyre_rps;
                    self.stupid_graphs.max_rotations_gear = self.current_gear;
                    self.stupid_graphs.max_rotations_rpm = rpm;
                }

                let mut ratio_struct = Ratio {
                    gear: self.current_gear,
                    acceleration,
                    ratio,
                    velocity: tyre_rps,
                    x_velocity,
                    rpm: rpm as f32,
                    differential,
                    inputs: inputs.clone(),
                };

                let entry = self.stupid_graphs
                    .ratios
                    .entry(self.current_gear)
                    .or_insert(ratio_struct.clone());

                if inputs.throttle > 0.2 && inputs.clutch <= f32::EPSILON
                    && inputs.brake <= f32::EPSILON
                    && (differential <= entry.differential)
                // && entry.ratio > ratio
                {
                    std::mem::swap(entry, &mut ratio_struct);
                }
            }

            // if timer::get_ticks(_ctx) % 1000 == 0 {
            //     println!("Stupid_data: {:?}", self.stupid_graphs);
            // }
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

        let power_scale = Point2::new(x_scale, y_scale);

        self.power_data.throttle.draw(
            ctx,
            throttle_color,
            throttle_color,
            self.screen_height,
            Point2::new(x_scale, self.screen_height * 0.95),
        )?;
        self.power_data.torque.draw(
            ctx,
            torque_color,
            torque_dot,
            self.screen_height,
            power_scale.clone(),
        )?;
        self.power_data.power.draw(
            ctx,
            hp_color,
            hp_dot,
            self.screen_height,
            power_scale.clone(),
        )?;

        //net
        {
            graphics::set_color(ctx, Color::from_rgba(127, 127, 127, 127))?;

            for rpm_vert in (0..self.max_rpm as u32).step_by(1000) {
                let x = rpm_vert as f32 * (self.screen_width / self.max_rpm as f32);

                if rpm_vert == 1000 {
                    let text = self.numeric_text_cache.numbers.get(&1000).unwrap();
                    let dest = Point2::new(
                        x + 2f32,
                        self.screen_height - (text.height() as f32 / 2f32) - 2f32,
                    );

                    graphics::draw_ex(
                        ctx,
                        text,
                        DrawParam {
                            dest,
                            scale: Point2::new(0.5, 0.5),
                            ..Default::default()
                        },
                    )?;
                }

                graphics::line(
                    ctx,
                    &[
                        Point2::new(x, self.screen_height * 0.05),
                        Point2::new(x, self.screen_height),
                    ],
                    1f32,
                )?;
            }
            for horizontal in (0..(graph_height * 1.2) as u32).step_by(200) {
                let y = self.screen_height
                    - (horizontal as f32 * self.screen_height as f32 / graph_height);

                if horizontal == 200 {
                    let text = self.numeric_text_cache.numbers.get(&200).unwrap();
                    let dest = Point2::new(2f32, y - (text.height() as f32 / 2f32) - 2f32);
                    graphics::draw_ex(
                        ctx,
                        text,
                        DrawParam {
                            dest,
                            scale: Point2::new(0.5, 0.5),
                            ..Default::default()
                        },
                    )?;
                }

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
            self.stupid_graphs
                .ratios
                .get(&self.current_gear)
                .map(|a| a.ratio.clone())
                .unwrap_or(0f32) as i32,
            self.stupid_graphs.max_rotations as i32,
        ];

        self.stupid_graphs.draw(
            ctx,
            &self.power_data,
            &Point2::new(self.screen_width, self.screen_height),
            self.current_gear,
            self.max_rpm,
        )?;

        self.optimized_text
            .draw_num_cache(ctx, &values, &self.numeric_text_cache)?;

        graphics::present(ctx);

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

    pub fn draw_num_cache<'a>(
        &self,
        ctx: &mut Context,
        values: &[i32],
        cache: &NumericTextCache,
    ) -> GameResult<()> {
        let mut target = graphics::Point2::new(2f32, 2f32);
        graphics::set_color(ctx, WHITE)?;

        for (n, v) in self.names.iter().zip(values.iter()) {
            graphics::draw(ctx, n, target, 0f32)?;
            target.x += n.width() as f32;
            if let Some(value) = cache.numbers.get(v) {
                graphics::draw(ctx, value, target, 0f32)?;
                target.x += value.width() as f32 + 3f32;
            } else {
                // println!("No cached value: {}", v);
            }
        }
        Ok(())
    }
}
