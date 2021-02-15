trait DeepFlattenIteratorOf<Depth, T> {
    type DeepFlatten: Iterator<Item = T>;
    fn deep_flatten(this: Self) -> Self::DeepFlatten;
}

impl<I: Iterator> DeepFlattenIteratorOf<(), I::Item> for I {
    type DeepFlatten = Self;
    fn deep_flatten(this: Self) -> Self::DeepFlatten {
        this
    }
}

impl<Depth, I: Iterator, T> DeepFlattenIteratorOf<(Depth,), T> for I
where
    std::iter::Flatten<I>: DeepFlattenIteratorOf<Depth, T>,
    I: Iterator,
    <I as Iterator>::Item: IntoIterator,
{
    type DeepFlatten = <std::iter::Flatten<I> as DeepFlattenIteratorOf<Depth, T>>::DeepFlatten;
    fn deep_flatten(this: Self) -> Self::DeepFlatten {
        DeepFlattenIteratorOf::deep_flatten(this.flatten())
    }
}

// wrapper type to help out type inference
struct DeepFlatten<Depth, I, T>
where
    I: DeepFlattenIteratorOf<Depth, T>,
{
    inner: I::DeepFlatten,
}

/// Recursively flattens an iterator
/// Shamelessly taken from https://users.rust-lang.org/t/trying-to-make-a-recursive-flatten-function/50059
trait DeepFlattenExt: Iterator + Sized {
    fn deep_flatten<Depth, T>(self) -> DeepFlatten<Depth, Self, T>
    where
        Self: DeepFlattenIteratorOf<Depth, T>,
    {
        DeepFlatten {
            inner: DeepFlattenIteratorOf::deep_flatten(self),
        }
    }
}
impl<I: Iterator> DeepFlattenExt for I {}
impl<Depth, I, T> Iterator for DeepFlatten<Depth, I, T>
where
    I: DeepFlattenIteratorOf<Depth, T>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use super::DeepFlattenExt;

    #[test]
    fn empty() {
        let input: &[i32] = &[];
        let flattened = input.into_iter().deep_flatten();
        // Collect the iterator's items into a container.
        let collected: Vec<i32> = flattened.copied().collect();
        assert_eq!(collected.len(), 0); // This really just tests that it runs without crashing.
    }

    #[test]
    fn identity() {
        let input = vec![1, 2, 3];
        let expected = vec![1, 2, 3];

        let flattened = input.into_iter().deep_flatten();
        let collected: Vec<i32> = flattened.collect();
        assert_eq!(collected.len(), 3);

        assert!(collected.iter().eq(expected.iter()));
    }

    #[test]
    fn one_level() {
        let input = vec![vec![1, 2], vec![3, 4]];
        let expected = vec![1, 2, 3, 4];

        // This is something that just std::iter::Flatten can do.
        // If there are more than one level involved, you have to specify the desired type.
        let flattened = input.into_iter().deep_flatten::<_, i32>();
        let collected: Vec<i32> = flattened.collect();
        assert_eq!(collected.len(), 4);

        assert!(collected.iter().eq(expected.iter()));
    }

    #[test]
    fn two_levels() {
        let input = vec![vec![vec![1, 2], vec![3, 4]], vec![vec![5, 6], vec![7, 8]]];
        let expected = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let flattened = input.into_iter().deep_flatten::<_, i32>();
        let collected: Vec<i32> = flattened.collect();
        assert_eq!(collected.len(), 8);

        assert!(collected.iter().eq(expected.iter()));
    }

    #[test]
    fn depth() {
        let input = vec![vec![vec![1, 2], vec![3, 4]], vec![vec![5, 6], vec![7, 8]]];
        let expected = vec![vec![1, 2], vec![3, 4], vec![5, 6], vec![7, 8]];

        let flattened = input.into_iter().deep_flatten::<_, Vec<i32>>();
        let collected: Vec<Vec<i32>> = flattened.collect();
        assert_eq!(collected.len(), 4);

        assert!(collected.iter().eq(expected.iter()));
    }
}
