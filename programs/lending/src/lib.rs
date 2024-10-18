pub mod constants;
pub mod error;
pub mod contexts;
pub mod states;

use anchor_lang::prelude::*;

pub use constants::*;
pub use contexts::*;
pub use states::*;

declare_id!("8iZGbJw7yWA4znvCcnz4VGKhdnzRwGPiU5BjLpV539Kc");

#[program]
pub mod lending {
    use super::*;
}
