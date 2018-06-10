pub mod data;

use ggez::graphics::*;
use ggez::*;
use std::cmp::Ordering;
use std::collections::HashSet;
use strsim;

pub struct AllCarsData {
    data: Vec<CarInfoCached>,
    current_car_ix: Option<usize>,
    font: Font,
}

impl AllCarsData {
    pub fn new(font: Font) -> AllCarsData {
        let data = data::create().into_iter().map(|d| d.with_cache()).collect();

        AllCarsData {
            data,
            current_car_ix: None,
            font,
        }
    }

    pub fn set(&mut self, car_name: &String) {
        let car_info = self.data
            .iter()
            .enumerate()
            .max_by(|(_, info_l), (_, info_r)| {
                let sl = strsim::jaro_winkler(&info_l.full_name, &car_name);
                let sr = strsim::jaro_winkler(&info_r.full_name, &car_name);

                if sl > sr {
                    Ordering::Greater
                } else if sl == sr {
                    inclusions(&car_name, &info_l.full_name)
                        .cmp(&inclusions(&car_name, &info_r.full_name))
                } else {
                    Ordering::Less
                }
            })
            .unwrap();

        self.current_car_ix = Some(car_info.0);
    }

    pub fn good_name(&self) -> Option<String> {
        if let Some(ix) = self.current_car_ix {
            Some(self.data[ix].full_name.clone())
        } else {
            None
        }
    }

    pub fn draw_from_right(&mut self, ctx: &mut Context, dest: &Point2) -> GameResult<()> {
        if let Some(ix) = self.current_car_ix {
            let text: &Text = self.data[ix].get_text(ctx, &self.font);
            let dest = Point2::new(dest.x - text.width() as f32 - 2f32, dest.y + 2f32);
            text.draw(ctx, dest, 0f32)?
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct CarInfo {
    name: &'static str,
    model: &'static str,
    sd_vol: i32,
    sd_tone: i32,
    norm_vol: i32,
    norm_tone: i32,
    ads: Option<&'static str>,
}

#[derive(Clone)]
struct CarInfoCached {
    car_info: CarInfo,
    full_name: String,
    text: Option<Text>,
}

impl CarInfoCached {
    pub fn get_text<'a>(&'a mut self, ctx: &mut Context, font: &Font) -> &'a Text {
        if let Some(ref txt) = self.text {
            &txt
        } else {
            let rates = format!(
                "SD: {}/{}, REG: {}/{}{}{}",
                self.car_info.sd_vol,
                self.car_info.sd_tone,
                self.car_info.norm_vol,
                self.car_info.norm_tone,
                self.car_info.ads.map(|_| " ".into()).unwrap_or(""),
                self.car_info.ads.unwrap_or(""),
            );

            let text = Text::new(ctx, &rates, font).unwrap();
            self.text = Some(text);
            self.get_text(ctx, font)
        }
    }
}

impl CarInfo {
    fn new(
        name: &'static str,
        model: &'static str,
        sd_vol: i32,
        sd_tone: i32,
        norm_vol: i32,
        norm_tone: i32,
        ads: Option<&'static str>,
    ) -> CarInfo {
        CarInfo {
            name,
            model,
            sd_vol,
            sd_tone,
            norm_vol,
            norm_tone,
            ads,
        }
    }

    fn with_cache(self) -> CarInfoCached {
        let full_name = format!("{} {}", self.name, self.model);

        CarInfoCached {
            car_info: self,
            full_name,
            text: None,
        }
    }
}

fn inclusions(original: &str, check: &str) -> usize {
    let clean = original
        .chars()
        .map(|x| {
            if x.is_alphanumeric() || x.is_whitespace() {
                x
            } else {
                ' '
            }
        })
        .collect::<String>();

    let checks = check.split_whitespace().collect::<HashSet<_>>();

    clean
        .split_whitespace()
        .filter(|x| checks.contains(x))
        .count()
}
