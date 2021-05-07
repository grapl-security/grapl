pub trait AsStaticPython {
    fn as_static_python(&self) -> &'static str;
}

impl AsStaticPython for bool {
    fn as_static_python(&self) -> &'static str {
        match self {
            true => "True",
            false => "False",
        }
    }
}
