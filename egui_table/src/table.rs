use egui::{Id, IdMap, NumExt as _, Ui, Vec2, Vec2b};

use crate::{columns::Column, SplitScroll, SplitScrollDelegate};

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct TableState {
    // Maps columns ids to their widths.
    pub col_widths: IdMap<f32>,
}

pub struct Table {
    pub columns: Vec<Column>,

    pub id_salt: Id,

    /// Which columns are sticky (non-scrolling)?
    pub num_sticky_cols: usize,

    /// The count and heights of the sticky (non-scrolling) rows.
    /// Usually used for a single header row.
    pub sticky_row_heights: Vec<f32>,

    /// Height of the non-sticky rows.
    pub row_height: f32,

    /// Total number of rows (sticky + non-sticky).
    pub num_rows: usize,
}

pub trait TableDelegate {}

impl Table {
    pub fn show(self, ui: &mut Ui, delegate: &mut dyn TableDelegate) {
        let Self {
            mut columns,
            id_salt,
            num_sticky_cols,
            sticky_row_heights,
            row_height,
            num_rows,
        } = self;

        let num_sticky_cols = num_sticky_cols.at_most(columns.len());
        let num_rows = num_rows.at_least(sticky_row_heights.len());
        let num_scroll_rows = num_rows - sticky_row_heights.len();

        let id = ui.make_persistent_id(id_salt);
        let mut state: TableState = ui.data_mut(|d| d.get_persisted(id)).unwrap_or_default();

        for (i, column) in columns.iter_mut().enumerate() {
            let column_id = Id::new(i); // TODO(emilk): let users set column ids
            if let Some(existing_width) = state.col_widths.get(&column_id) {
                column.current = *existing_width;
            }
        }

        Column::auto_size(&mut columns, ui.available_width());

        let sticky_size = Vec2::new(
            columns[..num_sticky_cols].iter().map(|c| c.current).sum(),
            sticky_row_heights.iter().sum(),
        );

        struct TableSplitScrollDelegate {}

        impl SplitScrollDelegate for TableSplitScrollDelegate {
            fn left_top_ui(&mut self, ui: &mut Ui) {
                todo!()
            }

            fn right_top_ui(&mut self, ui: &mut Ui) {
                todo!()
            }

            fn left_bottom_ui(&mut self, ui: &mut Ui) {
                todo!()
            }

            fn right_bottom_ui(&mut self, ui: &mut Ui) {
                todo!()
            }
        }

        SplitScroll {
            scroll_enabled: Vec2b::new(true, true),
            fixed_size: sticky_size,
            scroll_outer_size: (ui.available_size() - sticky_size).at_least(Vec2::ZERO),
            scroll_content_size: Vec2::new(
                columns[num_sticky_cols..].iter().map(|c| c.current).sum(),
                row_height * num_scroll_rows as f32,
            ),
        }
        .show(ui, &mut TableSplitScrollDelegate {});
    }
}
