//! Run this file with `cargo test --test 04_merge_slices`.

// TODO: Implement a function called `merge_slices`, which is useful for the merge sort algorithm.
// It will take two sorted `u32` slices as inputs and merge them into a sorted vector (Vec).
// The function will return the vector.
// Bonus: Can you build a complete merge sort on top of this function? :)

fn merge_slices(vec1: &[i32], vec2: &[i32]) -> Vec<i32> {
    // let mut result = Vec::new();
    // let mut i = 0;
    // let mut j = 0;

    // while i < vec1.len() && j < vec2.len() {
    //     if vec1[i] < vec2[j] {
    //         result.push(vec1[i]);
    //         i += 1;
    //     } else {
    //         result.push(vec2[j]);
    //         j += 1;
    //     }
    // }

    // while i < vec1.len() {
    //     result.push(vec1[i]);
    //     i += 1;
    // }

    // while j < vec2.len() {
    //     result.push(vec2[j]);
    //     j += 1;
    // }

    // return result;

    // Schválně jsem si dal práci udělat to jen iterátorama. Nooooo NE, ale aspoň jsem objevil peekable()
    let mut vec1 = vec1.iter().peekable();
    let mut vec2 = vec2.iter().peekable();
    let mut result = Vec::new();

    loop {
        match (vec1.peek(), vec2.peek()) {
            (None, None) => return result,
            (None, Some(_)) => {
                result.extend(vec2);
                return result
            },
            (Some(_), None) => {
                result.extend(vec1);
                return result
            },
            (Some(a), Some(b)) => {
                match a.cmp(b) {
                    std::cmp::Ordering::Less => {
                        result.push(*vec1.next().unwrap());
                    }
                    std::cmp::Ordering::Greater => {
                        result.push(*vec2.next().unwrap());
                    }
                    std::cmp::Ordering::Equal => {
                        result.push(*vec1.next().unwrap());
                        result.push(*vec2.next().unwrap());
                    }
                }
            }
        }
    };
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use crate::merge_slices;

    #[test]
    fn merge_slices_empty() {
        assert_eq!(merge_slices(&[], &[]), vec![]);
    }

    #[test]
    fn merge_slices_basic() {
        assert_eq!(merge_slices(&[1, 2, 3], &[4, 5, 6]), vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn merge_slices_interleaved() {
        assert_eq!(merge_slices(&[1, 3, 5], &[2, 4, 6]), vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn merge_slices_duplicates() {
        assert_eq!(merge_slices(&[1, 1, 3], &[1, 3, 4]), vec![1, 1, 1, 3, 3, 4]);
    }

    #[test]
    fn merge_slices_uneven_size() {
        assert_eq!(
            merge_slices(&[1, 4, 6, 8], &[0, 1, 1, 3, 4, 5, 7, 8, 9]),
            vec![0, 1, 1, 1, 3, 4, 4, 5, 6, 7, 8, 8, 9]
        );
    }

    #[test]
    fn merge_slices_first_empty() {
        assert_eq!(merge_slices(&[], &[1, 4, 8]), vec![1, 4, 8]);
    }

    #[test]
    fn merge_slices_second_empty() {
        assert_eq!(merge_slices(&[1, 9, 11], &[]), vec![1, 9, 11]);
    }
}
