use std::collections::HashMap;

use ggez::graphics::*;
use ggez::*;
use util::*;

pub struct NetsAndBorders {
    region_borders: Mesh,
    #[allow(dead_code)]
    cm_text: HashMap<i32, Text>,
    left_region_horizontal: Mesh,
    left_region_vertical: Mesh,
    right_region_horizontal: Mesh,
    right_region_vertical: Mesh,
    one_g_text: Text,
    fifty_kmph_text: Text,
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
        let cm_text = (-50..50)
            .step_by(1)
            .map(|num| {
                (
                    num,
                    graphics::Text::new(ctx, &format!("{} cm", num), font).unwrap(),
                )
            })
            .collect();

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

        let one_g_text = Text::new(ctx, "1g", font).unwrap();
        let fifty_kmph_text = Text::new(ctx, "500m", font).unwrap();

        NetsAndBorders {
            region_borders,
            cm_text,
            left_region_horizontal,
            left_region_vertical,
            right_region_vertical,
            right_region_horizontal,
            one_g_text,
            fifty_kmph_text,
        }
    }

    pub fn draw(
        &self,
        ctx: &mut Context,
        max_x: i32,
        max_y: f32,
        max_distance: f32,
        screen_size: &Point2,
        numeric_cache: &NumericTextCache,
        max_rh: f32,
        min_rh: f32,
    ) -> GameResult<()> {
        let draw_digit =
            |ctx: &mut Context, mut point: Point2, digit: i32, fat: bool| -> GameResult<()> {
                let text = numeric_cache.small_numbers.get(&digit);

                if let Some(text) = text {
                    point.x += if fat { 4f32 } else { 3f32 };
                    point.y -= text.height() as f32 + 2f32;

                    graphics::draw_ex(
                        ctx,
                        text,
                        DrawParam {
                            dest: point,
                            ..Default::default()
                        },
                    )?
                }
                Ok(())
            };

        let draw_cm =
            |ctx: &mut Context, mut point: Point2, number: i32, fat: bool| -> GameResult<()> {
                let text = self.cm_text.get(&number);
                if let Some(text) = text {
                    point.x += if fat { 4f32 } else { 3f32 };
                    point.y += 2f32;
                    graphics::draw_ex(
                        ctx,
                        text,
                        DrawParam {
                            dest: point,
                            ..Default::default()
                        },
                    )?;
                }
                Ok(())
            };

        graphics::set_color(ctx, Color::from_rgba(127, 127, 127, 127))?;
        self.region_borders
            .draw(ctx, Point2::new(0f32, 0f32), 0f32)?;

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

        for rh in ((min_rh * 1.2) as i32)..(max_rh * 1.2) as i32 {
            let y = (rh as f32 - min_rh * 1.2) / (max_rh * 1.2 - min_rh * 1.2);
            let dest = scale_right_bottom(0f32, y, screen_size);
            if rh % 2 != 0 || rh == 0 {
                draw_cm(ctx, dest, rh, true)?;
            }

            graphics::draw(ctx, &self.right_region_horizontal, dest, 0f32)?;
        }

        for accel in 1..11 {
            let y = accel as f32 / 10f32;
            let dest = scale_left(0f32, y, screen_size);
            if accel == 1 {
                let mut dest = scale_left(0f32, y, screen_size);
                dest.x += 3f32;
                dest.y -= self.one_g_text.height() as f32 + 2f32;
                self.one_g_text.draw_ex(
                    ctx,
                    DrawParam {
                        dest,
                        ..Default::default()
                    },
                )?;
            }

            graphics::draw(ctx, &self.left_region_horizontal, dest, 0f32)?;
        }

        for distance in (0..max_distance as i32).step_by(500) {
            let x = distance as f32 / max_distance;
            if distance == 500 {
                let mut dest = scale_left(x, 0f32, screen_size);
                dest.x += 3f32;
                dest.y -= self.fifty_kmph_text.height() as f32 + 2f32;
                self.fifty_kmph_text.draw_ex(
                    ctx,
                    DrawParam {
                        dest,
                        ..Default::default()
                    },
                )?;
            }
            let dest = scale_left(x, 1f32, screen_size);

            graphics::draw(ctx, &self.left_region_vertical, dest, 0f32)?;
        }

        Ok(())
    }
}

#[inline]
pub fn scale_left(x: f32, y: f32, scr_size: &Point2) -> Point2 {
    Point2::new(
        0.6 * x * scr_size.x,
        (scr_size.y * 0.05) + (1.0 - y) * (scr_size.y * 0.95),
    )
}

#[inline]
pub fn scale_right_top(x: f32, y: f32, scr_size: &Point2) -> Point2 {
    Point2::new(
        (0.6 + x * 0.4) * scr_size.x,
        (scr_size.y * 0.05) + (1.0 - y) * (scr_size.y * 0.475),
    )
}

#[inline]
pub fn scale_right_bottom(x: f32, y: f32, scr_size: &Point2) -> Point2 {
    Point2::new(
        (0.6 + x * 0.4) * scr_size.x,
        (scr_size.y * 0.525) + (1.0 - y) * (scr_size.y * 0.475),
    )
}
