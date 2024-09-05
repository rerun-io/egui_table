use std::collections::BTreeMap;

use egui::{vec2, Id, IdMap, NumExt as _, Rect, Ui, UiBuilder, Vec2, Vec2b};
use vec1::Vec1;

use crate::{columns::Column, SplitScroll, SplitScrollDelegate};

// TODO: fix the functionality of this
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum AutoSizeMode {
    /// Never auto-size the columns.
    #[default]
    Never,

    /// Always auto-size the columns
    Always,

    /// Auto-size the columns if the parents' width changes
    OnParentResize,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct TableState {
    // Maps columns ids to their widths.
    pub col_widths: IdMap<f32>,

    pub parent_width: Option<f32>,
}

impl TableState {
    pub fn load(ctx: &egui::Context, id: Id) -> Option<Self> {
        ctx.data_mut(|d| d.get_persisted(id))
    }

    pub fn store(self, ctx: &egui::Context, id: Id) {
        ctx.data_mut(|d| d.insert_persisted(id, self));
    }

    pub fn id(ui: &Ui, id_salt: Id) -> Id {
        ui.make_persistent_id(id_salt)
    }

    pub fn reset(ctx: &egui::Context, id: Id) {
        ctx.data_mut(|d| {
            d.remove::<Self>(id);
        });
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Header {
    height: f32,
}

impl Header {
    pub fn new(height: f32) -> Self {
        Self { height }
    }
}

/// A table viewer.
///
/// Designed to be fast when there are millions of rows, but only hundreds of columns.
///
/// ## Sticky columns and rows
/// You can designate a certain number of column and rows as being "sticky".
/// These won't scroll with the rest of the table.
///
/// The sticky rows are always the first ones at the top, and are usually used for the column headers.
/// The sticky columns are always the first ones on the left, useful for special columns like
/// table row number or similar.
/// A sticky column is sometimes called a "gutter".
///
/// ## Batteries not included
/// * You need to specify the `Table` size beforehand
/// * Does not add any margins to cells. Add it yourself with [`egui::Frame`].
/// * Does not wrap cells in scroll areas. Do that yourself.
/// * Doesn't paint any guide-lines for the rows. Paint them yourself.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Table {
    pub columns: Vec<Column>,

    pub id_salt: Id,

    /// Which columns are sticky (non-scrolling)?
    pub num_sticky_cols: usize,

    /// The count and parameters of the sticky (non-scrolling) header rows.
    pub headers: Vec<Header>,

    /// Height of the non-sticky rows.
    pub row_height: f32,

    /// Total number of rows (sticky + non-sticky).
    pub num_rows: u64,

    pub auto_size_mode: AutoSizeMode,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            columns: vec![],
            id_salt: Id::new("table"),
            num_sticky_cols: 1,
            headers: vec![Header::new(16.0)],
            row_height: 16.0,
            num_rows: 0,
            auto_size_mode: AutoSizeMode::default(),
        }
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct CellInfo {
    pub col_nr: usize,

    pub row_nr: u64,
    // We could add more stuff here, like a reference to the column
}

pub trait TableDelegate {
    /// Called before any call to [`Self::cell_ui`] to prefetch the range of visible rows.
    fn prefetch_rows(&mut self, _row_numbers: std::ops::Range<u64>) {}

    /// The contents of a header cell in the table.
    ///
    /// The [`CellInfo::row_nr`] is which header row (usually 0).
    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo);

    /// The contents of a cell in the table.
    ///
    /// The [`CellInfo::row_nr`] is ignoring header rows.
    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo);
}

impl Table {
    pub fn show(mut self, ui: &mut Ui, table_delegate: &mut dyn TableDelegate) {
        self.num_sticky_cols = self.num_sticky_cols.at_most(self.columns.len());
        self.num_rows = self.num_rows.at_least(self.headers.len() as u64);
        let num_scroll_rows = self.num_rows - self.headers.len() as u64;

        let id = TableState::id(ui, self.id_salt);
        let state = TableState::load(ui.ctx(), id);
        let is_new = state.is_none();
        let do_full_sizing_pass = is_new;
        let mut state = state.unwrap_or_default();

        for (i, column) in self.columns.iter_mut().enumerate() {
            let column_id = column.id(i);
            if let Some(existing_width) = state.col_widths.get(&column_id) {
                column.current = *existing_width;
            }
            column.current = column.range.clamp(column.current);

            if do_full_sizing_pass {
                column.auto_size_this_frame = true;
            }
        }

        let parent_width = ui.available_width();
        let auto_size = match self.auto_size_mode {
            AutoSizeMode::Never => false,
            AutoSizeMode::Always => true,
            AutoSizeMode::OnParentResize => state.parent_width.map_or(true, |w| w != parent_width),
        };
        if auto_size {
            Column::auto_size(&mut self.columns, parent_width);
        }
        state.parent_width = Some(parent_width);

        let col_x = {
            let mut x = ui.cursor().min.x;
            let mut col_x = Vec1::with_capacity(x, self.columns.len() + 1);
            for column in &self.columns {
                x += column.current;
                col_x.push(x);
            }
            col_x
        };

        let header_row_y = {
            let mut y = ui.cursor().min.y;
            let mut sticky_row_y = Vec1::with_capacity(y, self.headers.len() + 1);
            for header in &self.headers {
                y += header.height;
                sticky_row_y.push(y);
            }
            sticky_row_y
        };

        let sticky_size = Vec2::new(
            self.columns[..self.num_sticky_cols]
                .iter()
                .map(|c| c.current)
                .sum(),
            self.headers.iter().map(|h| h.height).sum(),
        );

        let mut ui_builder = UiBuilder::new();
        if do_full_sizing_pass {
            ui_builder = ui_builder.sizing_pass().invisible();
        }
        ui.scope_builder(ui_builder, |ui| {
            // Don't wrap text in the table cells.
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend); // TODO: I think this is default for horizontal layouts anyway?

            let num_columns = self.columns.len();

            table_delegate.prefetch_rows(0..self.headers.len() as u64);

            for (col_nr, column) in self.columns.iter_mut().enumerate() {
                if column.resizable {
                    let column_resize_id = id.with(column.id(col_nr)).with("resize");
                    if let Some(response) = ui.ctx().read_response(column_resize_id) {
                        if response.double_clicked() {
                            column.auto_size_this_frame = true;
                        }
                    }
                }
            }

            SplitScroll {
                scroll_enabled: Vec2b::new(true, true),
                fixed_size: sticky_size,
                scroll_outer_size: (ui.available_size() - sticky_size).at_least(Vec2::ZERO),
                scroll_content_size: Vec2::new(
                    self.columns[self.num_sticky_cols..]
                        .iter()
                        .map(|c| c.current)
                        .sum(),
                    self.row_height * num_scroll_rows as f32,
                ),
            }
            .show(
                ui,
                &mut TableSplitScrollDelegate {
                    id,
                    table_delegate,
                    state: &mut state,
                    table: &mut self,
                    col_x,
                    header_row_y,
                    max_column_widths: vec![0.0; num_columns],
                    visible_column_lines: Default::default(),
                    do_full_sizing_pass,
                    has_prefetched: false,
                },
            );
        });

        state.store(ui.ctx(), id);
    }
}

#[derive(Clone, Copy, Debug)]
struct ColumnResizer {
    offset: Vec2,
}

struct TableSplitScrollDelegate<'a> {
    id: Id,
    table_delegate: &'a mut dyn TableDelegate,
    table: &'a mut Table,
    state: &'a mut TableState,

    /// The x coordinate for the start of each column, plus the end of the last column.
    col_x: Vec1<f32>,

    /// The y coordinate for the start of each header row, plus the end of the last header row.
    header_row_y: Vec1<f32>,

    /// Actual width of the widest element in each column
    max_column_widths: Vec<f32>,

    /// Key is column number. The resizer is to the right of the column.
    visible_column_lines: BTreeMap<usize, ColumnResizer>,

    do_full_sizing_pass: bool,

    has_prefetched: bool,
}

impl<'a> TableSplitScrollDelegate<'a> {
    fn col_idx_at(&self, x: f32) -> usize {
        self.col_x
            .partition_point(|&col_x| col_x < x)
            .saturating_sub(1)
    }

    fn header_row_idx_at(&self, y: f32) -> u64 {
        self.header_row_y
            .partition_point(|&row_y| row_y < y)
            .saturating_sub(1) as u64
    }

    fn row_idx_at(&self, y: f32) -> u64 {
        let y = y - self.header_row_y.last();
        let row_nr = (y / self.table.row_height).floor() as u64;
        row_nr.at_most(self.table.num_rows.saturating_sub(1))
    }

    fn header_ui(&mut self, ui: &mut Ui, offset: Vec2) {
        // Find the visible range of columns and rows:
        let viewport = ui.clip_rect().translate(offset);

        let (first_col, last_col) = if self.do_full_sizing_pass {
            // We do the UI for all columns during a sizing pass, so we can auto-size ALL columns
            (0, self.col_x.len() - 1)
        } else {
            // Only paint the visible columns, as an optimization
            (
                self.col_idx_at(viewport.min.x),
                self.col_idx_at(viewport.max.x),
            )
        };
        let first_row = self.header_row_idx_at(viewport.min.y);
        let last_row = self.header_row_idx_at(viewport.max.y);

        for row_nr in first_row..=last_row {
            if self.table.num_rows <= row_nr {
                break;
            }
            for col_nr in first_col..=last_col {
                let Some(column) = self.table.columns.get(col_nr) else {
                    continue;
                };

                let mut cell_rect = Rect::from_x_y_ranges(
                    self.col_x[col_nr]..=self.col_x[col_nr + 1],
                    self.header_row_y[row_nr as usize]..=self.header_row_y[row_nr as usize + 1],
                )
                .translate(-offset);
                let clip_rect = cell_rect; // Note: we shrink the cell rect when auto-sizing, but not the clip rect! This is to avoid flicker.
                if column.auto_size_this_frame {
                    cell_rect.max.x = cell_rect.min.x + column.range.min;
                }

                let mut ui_builder = UiBuilder::new()
                    .max_rect(cell_rect)
                    .id_salt((row_nr, col_nr))
                    .layout(egui::Layout::left_to_right(egui::Align::Center));
                if column.auto_size_this_frame {
                    ui_builder = ui_builder.sizing_pass();
                }
                let mut cell_ui = ui.new_child(ui_builder);
                cell_ui.shrink_clip_rect(clip_rect);

                self.table_delegate
                    .header_cell_ui(&mut cell_ui, &CellInfo { col_nr, row_nr });

                let width = &mut self.max_column_widths[col_nr];
                *width = width.max(cell_ui.min_size().x);
            }
        }

        // Save column lines for later interaction:
        for col_nr in first_col..=last_col {
            let Some(column) = self.table.columns.get(col_nr) else {
                continue;
            };
            if column.resizable {
                self.visible_column_lines
                    .insert(col_nr, ColumnResizer { offset });
            }
        }
    }

    fn region_ui(&mut self, ui: &mut Ui, offset: Vec2, do_prefetch: bool) {
        // Find the visible range of columns and rows:
        let viewport = ui.clip_rect().translate(offset);

        let (first_col, last_col) = if self.do_full_sizing_pass {
            // We do the UI for all columns during a sizing pass, so we can auto-size ALL columns
            (0, self.col_x.len() - 1)
        } else {
            // Only paint the visible columns, as an optimization
            (
                self.col_idx_at(viewport.min.x),
                self.col_idx_at(viewport.max.x),
            )
        };
        let first_row = self.row_idx_at(viewport.min.y);
        let last_row = self.row_idx_at(viewport.max.y);

        if do_prefetch {
            let row_range = first_row..last_row + 1;
            self.table_delegate.prefetch_rows(row_range.clone());
            self.has_prefetched = true;
        } else {
            debug_assert!(
                self.has_prefetched,
                "SplitScroll delegate methods called in unexpected order"
            );
        }

        for row_nr in first_row..=last_row {
            if self.table.num_rows <= row_nr {
                break;
            }
            for col_nr in first_col..=last_col {
                let Some(column) = self.table.columns.get(col_nr) else {
                    continue;
                };

                let mut cell_rect =
                    Rect::from_x_y_ranges(self.col_x[col_nr]..=self.col_x[col_nr + 1], {
                        let y = self.header_row_y.last() + row_nr as f32 * self.table.row_height;
                        y..=y + self.table.row_height
                    })
                    .translate(-offset);
                let clip_rect = cell_rect; // Note: we shrink the cell rect when auto-sizing, but not the clip rect! This is to avoid flicker.
                if column.auto_size_this_frame {
                    cell_rect.max.x = cell_rect.min.x + column.range.min;
                }

                let mut ui_builder = UiBuilder::new()
                    .max_rect(cell_rect)
                    .id_salt((row_nr, col_nr))
                    .layout(egui::Layout::left_to_right(egui::Align::Center));
                if column.auto_size_this_frame {
                    ui_builder = ui_builder.sizing_pass();
                }
                let mut cell_ui = ui.new_child(ui_builder);
                cell_ui.shrink_clip_rect(clip_rect);

                self.table_delegate
                    .cell_ui(&mut cell_ui, &CellInfo { col_nr, row_nr });

                let width = &mut self.max_column_widths[col_nr];
                *width = width.max(cell_ui.min_size().x);
            }
        }

        // Save column lines for later interaction:
        for col_nr in first_col..=last_col {
            let Some(column) = self.table.columns.get(col_nr) else {
                continue;
            };
            if column.resizable {
                self.visible_column_lines
                    .insert(col_nr, ColumnResizer { offset });
            }
        }
    }
}

impl<'a> SplitScrollDelegate for TableSplitScrollDelegate<'a> {
    fn left_top_ui(&mut self, ui: &mut Ui) {
        self.header_ui(ui, Vec2::ZERO);
    }

    fn right_top_ui(&mut self, ui: &mut Ui) {
        self.header_ui(ui, vec2(ui.clip_rect().min.x - ui.min_rect().min.x, 0.0));
    }

    fn left_bottom_ui(&mut self, ui: &mut Ui) {
        self.region_ui(
            ui,
            vec2(0.0, ui.clip_rect().min.y - ui.min_rect().min.y),
            false,
        );
    }

    fn right_bottom_ui(&mut self, ui: &mut Ui) {
        self.region_ui(ui, ui.clip_rect().min - ui.min_rect().min, true);
    }

    fn finish(&mut self, ui: &mut Ui) {
        // Paint column resize lines

        for (col_nr, ColumnResizer { offset }) in &self.visible_column_lines {
            let col_nr = *col_nr;
            let Some(column) = self.table.columns.get(col_nr) else {
                continue;
            };
            if !column.resizable {
                continue;
            }

            let column_id = column.id(col_nr);
            let used_width = column.range.clamp(self.max_column_widths[col_nr]);

            let column_width = self
                .state
                .col_widths
                .entry(column_id)
                .or_insert(column.current);

            if ui.is_sizing_pass() || column.auto_size_this_frame {
                // Shrink to fit the widest element in the column:
                *column_width = used_width;
            } else {
                // Grow to fit the widest element in the column:
                *column_width = column_width.max(used_width);
            }

            let column_resize_id = self.id.with(column.id(col_nr)).with("resize");

            let mut x = self.col_x[col_nr + 1] - offset.x; // Right side of the column
            let yrange = ui.clip_rect().y_range();
            let line_rect = egui::Rect::from_x_y_ranges(x..=x, yrange)
                .expand(ui.style().interaction.resize_grab_radius_side);

            let resize_response =
                ui.interact(line_rect, column_resize_id, egui::Sense::click_and_drag());

            if resize_response.dragged() {
                if let Some(pointer) = ui.ctx().pointer_latest_pos() {
                    let desired_new_width = *column_width + pointer.x - x;
                    let desired_new_width = column.range.clamp(desired_new_width);

                    // We don't want to shrink below the size that was actually used.
                    // However, we still want to allow content that shrinks when you try
                    // to make the column less wide, so we allow some small shrinkage each frame:
                    // big enough to allow shrinking over time, small enough not to look ugly when
                    // shrinking fails. This is a bit of a HACK around immediate mode.
                    // TODO: do something smarter by remembering success/failure to resize from one frame to the next.
                    let max_shrinkage_per_frame = 8.0;
                    let new_width =
                        desired_new_width.at_least(used_width - max_shrinkage_per_frame);
                    let new_width = column.range.clamp(new_width);
                    x += new_width - *column_width;
                    *column_width = new_width;

                    if new_width != desired_new_width {
                        ui.ctx().request_repaint(); // Get there faster
                    }
                }
            }

            let dragging_something_else =
                ui.input(|i| i.pointer.any_down() || i.pointer.any_pressed());
            let resize_hover = resize_response.hovered() && !dragging_something_else;

            if resize_hover || resize_response.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
            }

            let stroke = if resize_response.dragged() {
                ui.style().visuals.widgets.active.bg_stroke
            } else if resize_hover {
                ui.style().visuals.widgets.hovered.bg_stroke
            } else {
                // ui.visuals().widgets.inactive.bg_stroke
                ui.visuals().widgets.noninteractive.bg_stroke
            };

            ui.painter().vline(x, yrange, stroke);
        }
    }
}
