use egui_table::{AutoSizeMode, Column, Table, TableDelegate};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TableDemo {
    num_columns: usize,
    default_column: Column,
    auto_size_mode: AutoSizeMode,
}

impl Default for TableDemo {
    fn default() -> Self {
        Self {
            num_columns: 10,
            default_column: Column::new(100.0, 100.0..=200.0),
            auto_size_mode: AutoSizeMode::default(),
        }
    }
}

impl TableDelegate for TableDemo {
    fn cell_ui(&mut self, ui: &mut egui::Ui, row_nr: usize, col_nr: usize) {
        ui.label(format!("row={row_nr}, col={col_nr}"));
    }
}

impl TableDemo {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("settings").num_columns(2).show(ui, |ui| {
            ui.label("Columns");
            ui.add(egui::DragValue::new(&mut self.num_columns).speed(1.0));
            ui.end_row();

            ui.label("Default column width");
            ui.add(egui::DragValue::new(&mut self.default_column.current).speed(1.0));
            ui.end_row();

            ui.label("Column width range");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.default_column.range.min).speed(1.0));
                ui.label("to");
                ui.add(egui::DragValue::new(&mut self.default_column.range.max).speed(1.0));
            });
            ui.end_row();

            ui.label("Auto-size mode");
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.auto_size_mode, AutoSizeMode::Never, "Never");
                ui.radio_value(&mut self.auto_size_mode, AutoSizeMode::Always, "Always");
                ui.radio_value(
                    &mut self.auto_size_mode,
                    AutoSizeMode::OnParentResize,
                    "OnParentResize",
                );
            });
        });

        ui.separator();

        Table {
            columns: vec![self.default_column; self.num_columns],
            id_salt: egui::Id::new("table_demo"),
            num_sticky_cols: 1,
            sticky_row_heights: vec![20.0; 1],
            row_height: 16.0,
            num_rows: 100,
            auto_size_mode: self.auto_size_mode,
        }
        .show(ui, self);
    }
}
