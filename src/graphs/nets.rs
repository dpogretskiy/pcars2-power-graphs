use ggez::*;
use ggez::graphics::*;
use util::*;

pub struct NetsAndBorders {
    x_text: Text,
    x_line: Mesh,
    y_text: Text,
    y_line: Mesh,
    borders: Mesh,
}

impl NetsAndBorders {
    pub fn new(
        ctx: &mut Context,
        screen_size: &Point2,
        numeric_text_cache: &NumericTextCache,
    ) -> NetsAndBorders {
        let borders = graphics::MeshBuilder::new()
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

        let y_text = numeric_text_cache.numbers.get(&200).unwrap().clone();
        let x_text = numeric_text_cache.numbers.get(&1000).unwrap().clone();

        let x_line = Mesh::new_line(
            ctx,
            &[
                Point2::new(0f32, 0f32),
                Point2::new(screen_size.x * 0.6, 0f32),
            ],
            1f32,
        ).unwrap();

        let y_line = Mesh::new_line(
            ctx,
            &[
                Point2::new(0f32, screen_size.y),
                Point2::new(0f32, screen_size.y * 0.05),
            ],
            1f32,
        ).unwrap();

        NetsAndBorders {
            x_line,
            y_line,
            borders,
            x_text,
            y_text,
        }
    }

    pub fn draw(
        &self,
        ctx: &mut Context,
        max_x: i32,
        max_y: f32,
        screen_size: &Point2,
    ) -> GameResult<()> {
        graphics::set_color(ctx, Color::from_rgba(127, 127, 127, 127))?;
        for rpm_vert in (0..max_x as u32).step_by(1000) {
            let x = rpm_vert as f32 * (screen_size.x * 0.6 / max_x as f32);

            if rpm_vert == 1000 {
                let dest = Point2::new(
                    x + 2f32,
                    screen_size.y - (self.x_text.height() as f32 / 2f32) - 2f32,
                );
                graphics::draw_ex(
                    ctx,
                    &self.x_text,
                    DrawParam {
                        dest,
                        scale: Point2::new(0.5, 0.5),
                        ..Default::default()
                    },
                )?;
            }

            graphics::draw_ex(
                ctx,
                &self.y_line,
                DrawParam {
                    dest: Point2::new(x, 0f32),
                    ..Default::default()
                },
            )?;
        }

        for horizontal in (0..(max_y * 1.2) as u32).step_by(200) {
            let y = screen_size.y - (horizontal as f32 * screen_size.y / max_y);

            if horizontal == 200 {
                let dest = Point2::new(2f32, y - (self.x_text.height() as f32 / 2f32) - 2f32);
                graphics::draw_ex(
                    ctx,
                    &self.y_text,
                    DrawParam {
                        dest,
                        scale: Point2::new(0.5, 0.5),
                        ..Default::default()
                    },
                )?;
            }

            graphics::draw_ex(
                ctx,
                &self.x_line,
                DrawParam {
                    dest: Point2::new(0f32, y),
                    ..Default::default()
                },
            )?;
        }

        self.borders.draw(ctx, Point2::new(0f32, 0f32), 0f32)?;
        Ok(())
    }
}
