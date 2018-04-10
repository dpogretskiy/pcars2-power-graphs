use std::collections::HashMap;

use ggez::*;
use ggez::graphics::*;
use util::*;

pub struct NetsAndBorders {
    region_borders: Mesh,
    percents: HashMap<i32, Text>,
    left_region_horizontal: Mesh,
    left_region_vertical: Mesh,
    right_region_horizontal: Mesh,
    right_region_vertical: Mesh,
}

impl NetsAndBorders {
    pub fn new(ctx: &mut Context, screen_size: &Point2, font: &Font) -> NetsAndBorders {
        //graph regions
        let region_borders = graphics::MeshBuilder::new()
            .line(
                &[
                    Point2::new(0f32, screen_size.y * 0.05),
                    Point2::new(screen_size.x, screen_size.y * 0.05),
                ],
                3f32,
            )
            .line(
                &[
                    Point2::new(screen_size.x * 0.6, screen_size.y * 0.05),
                    Point2::new(screen_size.x * 0.6, screen_size.y),
                ],
                3f32,
            )
            .line(
                &[
                    Point2::new(
                        screen_size.x * 0.6,
                        (screen_size.y + (screen_size.y * 0.05)) / 2f32,
                    ),
                    Point2::new(
                        screen_size.x,
                        (screen_size.y + (screen_size.y * 0.05)) / 2f32,
                    ),
                ],
                3f32,
            )
            .build(ctx)
            .unwrap();

        //graph texts
        let percents = (0..101)
            .step_by(10)
            .map(|num| (num, graphics::Text::new(ctx, &format!("{}%", num), font).unwrap())).collect();

        //left region lines
        let left_region_horizontal = Mesh::new_line(
            ctx,
            &[
                //left to right
                Point2::new(0f32, 0f32),
                Point2::new(screen_size.x * 0.6, 0f32),
            ],
            1f32,
        ).unwrap();

        let left_region_vertical = Mesh::new_line(
            ctx,
            &[
                Point2::new(0f32, 0f32),
                Point2::new(0f32, screen_size.y * 0.95),
            ],
            1f32,
        ).unwrap();
        //right regions lines
        let right_region_horizontal = Mesh::new_line(
            ctx,
            &[
                //left to right
                Point2::new(0f32, 0f32),
                Point2::new(screen_size.x * 0.4, 0f32),
            ],
            1f32,
        ).unwrap();

        let right_region_vertical = Mesh::new_line(
            ctx,
            &[
                Point2::new(0f32, 0f32),
                Point2::new(0f32, screen_size.y * 0.475),
            ],
            1f32,
        ).unwrap();

        NetsAndBorders {
            region_borders,
            percents,
            left_region_horizontal,
            left_region_vertical,
            right_region_vertical,
            right_region_horizontal,
        }
    }

    pub fn draw(
        &self,
        ctx: &mut Context,
        max_x: i32,
        max_y: f32,
        screen_size: &Point2,
        numeric_cache: &NumericTextCache,
    ) -> GameResult<()> {
        let draw_digit = |ctx: &mut Context, mut point: Point2, digit: i32, fat: bool| -> GameResult<()> {
            let text = numeric_cache.numbers.get(&digit);

            if let Some(text) = text {
                point.x += if fat { 3f32 } else { 2f32 };
                point.y -= text.height() as f32 / 2f32 + 2f32;

                graphics::draw_ex(
                    ctx,
                    text,
                    DrawParam {
                        dest: point,
                        scale: Point2::new(0.5, 0.5),
                        ..Default::default()
                    },
                )?
            }
            Ok(())
        };

        let draw_percent = |ctx: &mut Context, mut point: Point2, number: i32, fat: bool| -> GameResult<()> {
            let text = self.percents.get(&number);
            if let Some(text) = text {
                point.x += if fat { 3f32 } else { 2f32 };
                point.y += 2f32;
                graphics::draw_ex(
                    ctx,
                    text,
                    DrawParam {
                        dest: point,
                        scale: Point2::new(0.5, 0.5),
                        ..Default::default()
                    },
                )?;
            }
            Ok(())
        };

        graphics::set_color(ctx, Color::from_rgba(127, 127, 127, 127))?;
        self.region_borders.draw(ctx, Point2::new(0f32, 0f32), 0f32)?;

        for rpm in (0..max_x).step_by(1000) {
            let x = rpm as f32 / max_x as f32;

            if rpm == 1000 {
                let dest = scale_right_top(x, 0f32, screen_size);
                draw_digit(ctx, dest, 1000, false)?;
            }

            graphics::draw_ex(
                ctx,
                &self.right_region_vertical,
                DrawParam {
                    dest: scale_right_top(x, 1f32, screen_size),
                    ..Default::default()
                },
            )?;
        }

        for power in (0..max_y as u32).step_by(200) {
            let y = power as f32 / max_y;
            let dest = scale_right_top(0f32, y, screen_size);

            if power == 200 {
                draw_digit(ctx, dest, 200, true)?;
            }

            graphics::draw_ex(
                ctx,
                &self.right_region_horizontal,
                DrawParam {
                    dest,
                    ..Default::default()
                },
            )?;
        }

        for diff in (10i32..101).step_by(10) {
            let y = diff as f32 / 100.0;
            let dest = scale_right_bottom(0f32, y, screen_size);
            if diff == 10 || diff == 50 || diff == 100 {
                draw_percent(ctx, dest, diff, true)?;
            }

            graphics::draw(
                ctx, 
                &self.right_region_horizontal,
                dest,
                0f32
            )?;
        }
        Ok(())
    }
}

pub fn scale_left(x: f32, y: f32, scr_size: &Point2) -> Point2 {
    Point2::new(
        0.6 * x * scr_size.x,
        (scr_size.y * 0.05) + (scr_size.y - y * scr_size.y * 0.95),
    )
}

pub fn scale_right_top(x: f32, y: f32, scr_size: &Point2) -> Point2 {
    Point2::new(
        (0.6 + x * 0.4) * scr_size.x,
        (scr_size.y * 0.05) + (1.0 - y) * (scr_size.y * 0.475),
    )
}

pub fn scale_right_bottom(x: f32, y: f32, scr_size: &Point2) -> Point2 {
    Point2::new(
        (0.6 + x * 0.4) * scr_size.x,
        (scr_size.y * 0.525) + (1.0 - y) * (scr_size.y * 0.475),
    )
}
