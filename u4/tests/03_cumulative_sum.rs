//! Run this file with `cargo test --test 03_cumulative_sum`.

//! TODO: Implement a struct called `CumulativeSum`, which will be generic over two types - a value
//! type and an iterator over these value types. `CumulativeSum` will itself serve as an iterator,
//! which will return a cumulative sum of the items from the input iterator.
//! E.g. `CumulativeSum::new(vec![1, 2, 3])` will iterate `1`, `3`, and `6`.
//!
//! Think about the various trait bounds that you will require for `CumulativeSum` to work.
//! What operations have to be supported by the two generic types?

use std::ops::Add;

struct CumulativeSum<T>
where
    T: Add<Output = T> + Default + Copy,
{
    input_iter: std::vec::IntoIter<T>,
    sum: T,
}

impl<T> CumulativeSum<T>
where
    T: Add<Output = T> + Default + Copy,
{
    fn new(input_iter: std::vec::IntoIter<T>) -> Self {
        CumulativeSum {
            input_iter,
            sum: T::default(),
        }
    }
}

impl<T> Iterator for CumulativeSum<T>
where
    T: Add<Output = T> + Default + Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.input_iter.next();
        match n {
            Some(num) => {
                self.sum = self.sum + num;
                Some(self.sum)
            }
            None => None,
        }
    }
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use crate::CumulativeSum;
    use std::ops::Add;

    #[test]
    fn empty() {
        assert_eq!(CumulativeSum::new(Vec::<u32>::new().into_iter()).count(), 0);
    }

    #[test]
    fn single() {
        assert_eq!(
            CumulativeSum::new(vec![1].into_iter()).collect::<Vec<_>>(),
            vec![1]
        );
    }

    #[test]
    fn simple_u32() {
        let result = CumulativeSum::new(vec![1, 2, 3].into_iter()).collect::<Vec<_>>();
        assert_eq!(result, vec![1, 3, 6]);
    }

    #[test]
    fn simple_vec() {
        #[derive(Default, Copy, Clone, PartialEq, Debug)]
        struct Vec2D {
            x: u32,
            y: u32,
        }

        impl Add for Vec2D {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                }
            }
        }

        let result = CumulativeSum::new(
            vec![
                Vec2D { x: 2, y: 6 },
                Vec2D { x: 4, y: 0 },
                Vec2D { x: 2, y: 2 },
                Vec2D { x: 5, y: 3 },
            ]
            .into_iter(),
        )
        .collect::<Vec<_>>();
        assert_eq!(
            result,
            vec![
                Vec2D { x: 2, y: 6 },
                Vec2D { x: 6, y: 6 },
                Vec2D { x: 8, y: 8 },
                Vec2D { x: 13, y: 11 }
            ]
        );
    }
}
