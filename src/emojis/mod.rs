pub mod running_cat;
pub mod sleeping_cat;

pub use running_cat::RunningCat;
pub use sleeping_cat::SleepingCat;

pub trait EmojiRenderer {
  fn get_emoji() -> char;
}
