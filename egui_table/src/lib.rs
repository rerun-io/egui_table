//! Table viewer for [egui](https://www.egui.rs/).

pub mod columns;
mod split_scroll;
mod table;

pub use columns::Column;
pub use split_scroll::{SplitScroll, SplitScrollDelegate};
pub use table::{AutoSizeMode, CellInfo, HeaderRow, Table, TableDelegate, TableState};
