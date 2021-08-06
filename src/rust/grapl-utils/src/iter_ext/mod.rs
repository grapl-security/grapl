pub trait GraplIterExt: Iterator {
    /// Provides an 'owned' variant to `Itertools::Itertools`'s `::chunks` method.
    /// Returns an iterator that contains Vec<I::Item> each, at most, the size of `chunk_size`
    /// While the length of the individual Vec's are guaranteed to be equal or less-than `chunk_size`
    /// the capacity is not; adding to a Vec may trigger an allocation.
    fn chunks_owned(self, chunk_size: usize) -> ChunkedIterator<Self>
    where
        Self: Sized + Iterator,
    {
        ChunkedIterator::new(self, chunk_size)
    }
}

pub struct ChunkedIterator<I>
where
    I: Iterator + Sized,
{
    inner: I,
    chunk_size: usize,
}

impl<I> ChunkedIterator<I>
where
    I: Iterator + Sized,
{
    pub fn new(inner: I, chunk_size: usize) -> Self {
        Self { inner, chunk_size }
    }
}

impl<I> Iterator for ChunkedIterator<I>
where
    I: Iterator,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            None => None,
            Some(item) if self.chunk_size == 1 => Some(vec![item]),
            Some(item) => {
                let mut next_chunk = Vec::with_capacity(self.chunk_size);
                next_chunk.push(item);

                for item in &mut self.inner {
                    next_chunk.push(item);

                    // we know that chunk_size must be larger than 1
                    if next_chunk.len() == self.chunk_size {
                        break;
                    }
                }

                Some(next_chunk)
            }
        }
    }
}

impl<I> GraplIterExt for I where I: Iterator + Sized {}
