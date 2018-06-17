use ggez::graphics::*;
use ggez::*;
use std::collections::{HashMap, VecDeque};

pub struct NumericTextCache {
    pub numbers: HashMap<i32, graphics::Text>,
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

pub struct SlidingIter<F, T>
where
    F: Iterator<Item = T>,
{
    acc: VecDeque<T>,
    underlying: F,
    by: usize,
    loose: bool,
}

pub trait IntoSliding<T>
where
    Self: Iterator<Item = T> + Sized,
{
    fn sliding(self, by: usize) -> SlidingIter<Self, T>;
    fn sliding_loose(self, by: usize) -> SlidingIter<Self, T>;
}

impl<T, F> IntoSliding<T> for F
where
    F: Iterator<Item = T> + Sized,
{
    fn sliding(self, by: usize) -> SlidingIter<F, T> {
        SlidingIter {
            acc: VecDeque::with_capacity(by),
            underlying: self,
            by,
            loose: false,
        }
    }

    fn sliding_loose(self, by: usize) -> SlidingIter<F, T> {
        SlidingIter {
            acc: VecDeque::with_capacity(by),
            underlying: self,
            by,
            loose: true,
        }
    }
}

impl<F: Iterator<Item = T>, T: Clone> Iterator for SlidingIter<F, T> {
    type Item = VecDeque<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if !self.loose {
            if self.acc.len() == 0 {
                for _ in 0..self.by {
                    if let Some(elem) = self.underlying.next() {
                        self.acc.push_back(elem);
                    }
                }

                if self.acc.len() == self.by {
                    return Some(self.acc.clone());
                } else {
                    return None;
                }
            } else {
                match self.underlying.next() {
                    Some(elem) => {
                        self.acc.pop_front();
                        self.acc.push_back(elem);
                        Some(self.acc.clone())
                    }
                    None => None,
                }
            }
        } else {
            match self.underlying.next() {
                Some(elem) => {
                    if self.acc.len() >= self.by {
                        self.acc.pop_front();
                    }
                    self.acc.push_back(elem);
                    return Some(self.acc.clone());
                }
                None => {
                    if self.acc.len() > 0 {
                        self.acc.pop_front();
                    }

                    if self.acc.is_empty() {
                        None
                    } else {
                        Some(self.acc.clone())
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use test::Bencher;
    use util::*;
    #[test]
    fn sliding_sum() {
        let sum_target = (1 + 2) + (2 + 3) + (3 + 4) + (4 + 5);

        let sum: usize = (1..6usize)
            .sliding(2)
            .map::<usize, _>(|i| i.iter().sum())
            .sum();
        assert_eq!(sum, sum_target);
    }

    #[test]
    fn sliding_loose_sum() {
        let sum_target = 1 + (1 + 2) + (2 + 3) + (3 + 4) + (4 + 5) + 5;
        let sum: usize = (1..6usize)
            .sliding_loose(2)
            .map::<usize, _>(|i| i.iter().sum())
            .sum();
        assert_eq!(sum, sum_target);
    }

    #[test]
    fn sliding_by_one() {
        let one: usize = (1..100usize)
            .sliding_loose(1)
            .map::<usize, _>(|i| i.iter().sum())
            .sum();
        let two: usize = (1..100usize)
            .sliding(1)
            .map::<usize, _>(|i| i.iter().sum())
            .sum();

        assert_eq!(one, two);
        assert_eq!(one, (1..100).sum::<usize>());
    }

    #[bench]
    fn bench_tight(b: &mut Bencher) {
        b.iter(|| {
            let x: usize = (1..100000usize)
                .sliding_loose(5)
                .map::<usize, _>(|i| i.iter().fold(0, |_, _| 0))
                .sum();
        });
    }

    #[bench]
    fn bench_loose(b: &mut Bencher) {
        b.iter(|| {
            let x: usize = (1..100000usize)
                .sliding(5)
                .map::<usize, _>(|i| i.iter().fold(0, |_, _| 0))
                .sum();
        });
    }
}
