//! # Tutorial
//!
//! Table of Contents

pub mod chapter_0;
pub mod chapter_1;
pub mod chapter_2;
pub mod chapter_3;
pub mod chapter_4;
pub mod chapter_5;
pub mod chapter_6;
pub mod chapter_7;

// Macro to generate the inner tutorial navigation links,
// with optional previous and next links.
macro_rules! tutorial_links {
    (
        $( previous: $prev_mod:ident $(,)? )?
        $( next: $next_mod:ident $(,)? )?
    ) => {
        $( pub use super::$prev_mod as previous; )?
        $( pub use super::$next_mod as next; )?
        pub use crate::_tutorial as table_of_contents;
    }
}
pub(self) use tutorial_links;
