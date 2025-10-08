mod token_stream;
mod vi_state;

pub use token_stream::TokenStream;
pub use vi_state::VIMotions;

#[derive(Default, Debug, PartialEq, Eq)]
pub enum VIMode {
    #[default]
    Normal,
    Insert,
    Visual,
}
