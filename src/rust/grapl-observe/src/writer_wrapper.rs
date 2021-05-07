use std::io::{
    stdout,
    Stdout,
    Write,
};

pub struct WriterWrapper<W>
where
    W: Write,
{
    backing_writer: W,
}

impl<W> WriterWrapper<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Self {
            backing_writer: writer,
        }
    }

    #[allow(dead_code)]
    /// Mostly for testing purposes
    pub fn release(self) -> W {
        self.backing_writer
    }
}

/// If Stdout is ever upgraded to Clone we can just derive(Clone)
/// the Vec U8 is strictly for testing
impl Clone for WriterWrapper<Vec<u8>> {
    fn clone(&self) -> Self {
        Self {
            backing_writer: self.backing_writer.clone(),
        }
    }
}

impl Clone for WriterWrapper<Stdout> {
    fn clone(&self) -> Self {
        Self {
            backing_writer: stdout(),
        }
    }
}

impl<W> AsRef<W> for WriterWrapper<W>
where
    W: Write,
{
    fn as_ref(&self) -> &W {
        &self.backing_writer
    }
}

impl<W> AsMut<W> for WriterWrapper<W>
where
    W: Write,
{
    fn as_mut(&mut self) -> &mut W {
        &mut self.backing_writer
    }
}

impl<W> Write for WriterWrapper<W>
where
    W: Write,
{
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.backing_writer.write(data)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.backing_writer.flush()
    }
}
