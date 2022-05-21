use clap::Parser;

pub trait ParserExt: Parser {
    /// The default parse() in Clap reads arguments from the CLI.
    /// This can cause some unexpected interop with Cargo Test --nocapture.
    /// Since we only use the environment variables, I'm jut short-circuiting
    /// that behavior.
    fn parse_from_env() -> Self {
        // P.S. I've added a request to clap-rs for this to become a fully
        // supported feature:
        // https://github.com/clap-rs/clap/issues/3741
        let iter = std::iter::empty::<&str>();
        Self::parse_from(iter)
    }
}

impl<T> ParserExt for T where T: Parser {}
