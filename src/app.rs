use definitions::*;
use std;
use ggez::*;
use ggez::graphics::*;
use std::f32;
use std::time::Instant;
use graphs::*;
use graphs::nets::*;
use util::*;

pub struct PC2App {
    shared_data: *const SharedMemory,
    local_copy: SharedMemory,
    start_time: Instant,
    rpm_step: i32,
    current_gear: i32,
    current_rpm: i32,
    max_rpm: i32,
    current_car: String,
    current_track: String,
    power_data: PowerGraphData,
    stupid_graphs: StupidGraphData,
    roll_graph: RollGraphData,
    diff_graph: DiffGraphData,
    optimized_text: OptimizedText,
    numeric_text_cache: NumericTextCache,
    nets_and_borders: NetsAndBorders,
    screen_width: f32,
    screen_height: f32,
}

impl PC2App {
    pub fn new(
        ctx: &mut Context,
        sm: *const SharedMemory,
        screen_width: f32,
        screen_height: f32,
        rpm_step: i32,
    ) -> PC2App {
        let font = PC2App::load_font(ctx);
        let fragments = vec![
            graphics::Text::new(ctx, "MAXHP: ", &font).unwrap(),
            graphics::Text::new(ctx, "MAXRPM: ", &font).unwrap(),
            graphics::Text::new(ctx, "GEAR: ", &font).unwrap(),
            graphics::Text::new(ctx, "RPM: ", &font).unwrap(),
            graphics::Text::new(ctx, "HP: ", &font).unwrap(),
            graphics::Text::new(ctx, "GR: ", &font).unwrap(),
            graphics::Text::new(ctx, "Y: ", &font).unwrap(),
        ];
        let numeric_text_cache = NumericTextCache::new(ctx, &font);
        let optimized_text = OptimizedText::new(fragments);

        let nets_and_borders = NetsAndBorders::new(
            ctx,
            &Point2::new(screen_width, screen_height),
            &numeric_text_cache,
        );

        // "MAXHP: {} MAXRPM: {}, GEAR: {}, RPM: {}, HP: {}",
        PC2App {
            shared_data: sm,
            start_time: Instant::now(),
            local_copy: unsafe { std::ptr::read_volatile(sm) },
            optimized_text,
            current_gear: 0,
            current_rpm: 0,
            max_rpm: 1,
            power_data: PowerGraphData::new(rpm_step),
            stupid_graphs: StupidGraphData::new(),
            roll_graph: RollGraphData {},
            diff_graph: DiffGraphData::new(),
            current_car: String::new(),
            current_track: String::new(),
            screen_width,
            screen_height,
            numeric_text_cache,
            rpm_step,
            nets_and_borders,
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
            self.power_data = PowerGraphData::new(self.rpm_step);
            self.stupid_graphs = StupidGraphData::new();

            let mut title = car_name;
            title.push_str(" @ ");
            title.push_str(&track_name);
            graphics::get_window_mut(_ctx).set_title(&title).unwrap();
        }

        let current_rpm_f32 = local_copy.mRpm;
        let current_rpm = current_rpm_f32 as i32;
        let rpm = current_rpm - current_rpm % self.rpm_step;
        self.current_rpm = current_rpm;
        let throttle = local_copy.mThrottle;
        let torque = local_copy.mEngineTorque;
        let power = (torque * current_rpm_f32 / 9548.8) / 0.7457;

        let currents_only = !(throttle > 0.9999 && local_copy.mClutch < 0.0001);

        self.power_data.throttle.add(rpm, throttle, currents_only);
        self.power_data.torque.add(rpm, torque, currents_only);
        self.power_data.power.add(rpm, power, currents_only);

        //stupid stuff
        if self.current_gear != local_copy.mGear {
            self.current_gear = local_copy.mGear;
        } else {
            if self.current_gear > 0 {
                let tyre_rps_arr = local_copy.mTyreRPS.clone();
                let left_wheel_rps = tyre_rps_arr.data[Tyre::TyreRearLeft as usize];
                let right_wheel_rps = tyre_rps_arr.data[Tyre::TyreRearRight as usize];

                let differential = (left_wheel_rps - right_wheel_rps).abs();

                let diff_percent = (left_wheel_rps.abs().min(right_wheel_rps.abs()))
                    / (left_wheel_rps.abs().max(right_wheel_rps.abs()));

                if local_copy.mCurrentTime != 0f32 {
                    self.diff_graph.add(diff_percent, self.start_time.elapsed());
                }

                let inputs = Inputs::from(&local_copy);
                let x_velocity = local_copy.mLocalVelocity.x;

                let tyre_rps = ((tyre_rps_arr.data[Tyre::TyreRearLeft as usize]
                    + tyre_rps_arr.data[Tyre::TyreRearRight as usize])
                    / 2f32)
                    .abs();

                let ratio = current_rpm_f32 / tyre_rps;

                let acceleration = -local_copy.mLocalAcceleration.z;

                if self.stupid_graphs.max_ratio < ratio {
                    self.stupid_graphs.max_ratio = ratio;
                }

                //we want it to be minimal ratio gear also, so some lifting is going on here :)
                let min_known_ratio = self.stupid_graphs
                    .ratios
                    .iter()
                    .min_by_key(|x| x.1.ratio as i32)
                    .map(|x| x.1.ratio.clone());

                if let Some(min_ratio) = min_known_ratio {
                    if self.stupid_graphs.max_rotations_gear != self.current_gear
                        && ratio < min_ratio
                    {
                        self.stupid_graphs.max_rotations = tyre_rps;
                        self.stupid_graphs.max_rotations_gear = self.current_gear;
                        self.stupid_graphs.max_rotations_rpm = current_rpm;
                    }
                } else if self.stupid_graphs.max_rotations < tyre_rps {
                    self.stupid_graphs.max_rotations = tyre_rps;
                    self.stupid_graphs.max_rotations_gear = self.current_gear;
                    self.stupid_graphs.max_rotations_rpm = current_rpm;
                }

                let mut ratio_struct = Ratio {
                    gear: self.current_gear,
                    acceleration,
                    ratio,
                    velocity: tyre_rps,
                    x_velocity,
                    rpm: current_rpm_f32,
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
                {
                    std::mem::swap(entry, &mut ratio_struct);
                }
            }
        }

        self.local_copy = local_copy;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, Color::from_rgb(18, 31, 52))?;
        graphics::clear(ctx);

        self.power_data
            .draw(ctx, self.screen_height, 600f32, self.max_rpm)?;
        //net

        let graph_height = self.power_data
            .power
            .max_value
            .max(self.power_data.torque.max_value) * 1.2;

        let screen_size = Point2::new(self.screen_width, self.screen_height);

        self.nets_and_borders
            .draw(ctx, self.max_rpm, graph_height, &screen_size)?;

        //text
        let world_y = if self.local_copy.mViewedParticipantIndex >= 0 {
            let vec = &self.local_copy.mParticipantInfo.data
                [self.local_copy.mViewedParticipantIndex as usize]
                .mWorldPosition;
            vec.y
        } else {
            0f32
        };

        let values = vec![
            self.power_data.power.max_value as i32,
            self.max_rpm,
            self.current_gear,
            self.current_rpm - (self.current_rpm % 10),
            self.power_data.power.current_value.1 as i32,
            self.stupid_graphs
                .ratios
                .get(&self.current_gear)
                .map(|a| a.ratio.clone())
                .unwrap_or(0f32) as i32,
            world_y as i32,
        ];

        self.stupid_graphs.draw(
            ctx,
            &self.power_data,
            &Point2::new(600f32, self.screen_height),
            self.current_gear,
            self.max_rpm,
        )?;

        self.optimized_text
            .draw_num_cache(ctx, &values, &self.numeric_text_cache)?;

        self.roll_graph.draw(ctx, &self.local_copy, &screen_size)?;

        self.diff_graph.draw(ctx, &screen_size)?;

        graphics::present(ctx);

        Ok(())
    }
}
