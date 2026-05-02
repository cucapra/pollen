use std::io::{Result, Write};

/// Something that can be consumed and serialized as bytes.
///
/// This trait is sorta like `std::fmt::Display`, but it consumes the thing
/// being printed. This can be useful for temporary structs that would be
/// expensive to keep around beyond a single emission.
pub trait Emit {
    /// Write the value a stream.
    fn emit(self, f: &mut impl Write) -> Result<()>;

    /// Write the value to stdout.
    fn print(self)
    where
        Self: Sized,
    {
        self.emit(&mut std::io::stdout().lock()).unwrap();
    }
}
