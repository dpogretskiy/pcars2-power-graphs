use ggez::graphics::*;
use ggez::*;
use std::collections::HashMap;

pub struct NumericTextCache {
    pub numbers: HashMap<i32, graphics::Text>,
    pub small_numbers: HashMap<i32, graphics::Text>,
}

impl NumericTextCache {
    pub fn new(
        ctx: &mut Context,
        font: &graphics::Font,
        small_font: &graphics::Font,
    ) -> NumericTextCache {
        let mut numbers = HashMap::new();
        let mut small_numbers = HashMap::new();

        for number in (-1500..1501).chain((1510..20001).step_by(10)) {
            let txt = graphics::Text::new(ctx, &number.to_string(), font).unwrap();
            let small_txt = graphics::Text::new(ctx, &number.to_string(), small_font).unwrap();
            numbers.insert(number, txt);
            small_numbers.insert(number, small_txt);
        }

        NumericTextCache {
            numbers,
            small_numbers,
        }
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
