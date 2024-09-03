use egui_table::{Column, Table, TableDelegate};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct TableDemo {}

impl TableDelegate for TableDemo {
    fn cell_ui(&mut self, ui: &mut egui::Ui, row_nr: usize, col_nr: usize) {
        ui.label(format!("row={row_nr}, col={col_nr}"));
    }
}

impl TableDemo {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        Table {
            columns: vec![Column::new(100.0, 100.0..=200.0); 10],
            id_salt: egui::Id::new("table_demo"),
            num_sticky_cols: 1,
            sticky_row_heights: vec![20.0; 1],
            row_height: 16.0,
            num_rows: 100,
        }
        .show(ui, self);
    }
}
