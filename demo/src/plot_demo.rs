use egui::{pos2, vec2, Color32, Rect, Stroke, Ui, UiBuilder, Vec2, Vec2b};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct PlotDemo {}

impl PlotDemo {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        _ = self;
        let mut delegate = PlotDemoPartialScrollDelegate {};

        SplitScroll {
            scroll_enabled: Vec2b::new(true, true),
            fixed_size: vec2(123.0, 37.0),
            scroll_outer_size: vec2(600.0, 400.0),
            scroll_content_size: vec2(10_000.0, 10_000.0),
        }
        .show(ui, &mut delegate);
    }
}

struct PlotDemoPartialScrollDelegate {}

impl PartialScrollDelegate for PlotDemoPartialScrollDelegate {
    fn left_top_ui(&mut self, ui: &mut Ui) {
        checkerboard(ui);
        ui.label("Fixed region");
    }

    fn right_top_ui(&mut self, ui: &mut Ui) {
        checkerboard(ui);
        ui.label("Horizontally scrollable");
    }

    fn left_bottom_ui(&mut self, ui: &mut Ui) {
        checkerboard(ui);
        ui.label("Vertically scrollable");
    }

    fn right_bottom_ui(&mut self, ui: &mut Ui) {
        checkerboard(ui);
        ui.label("Fully scrollable");
    }
}

fn checkerboard(ui: &Ui) {
    let rect = ui.max_rect();
    // ui.painter()
    //     .rect_stroke(rect.shrink(0.5), 1.0, (1.0, ui.visuals().text_color()));

    let fill_color = ui.visuals().faint_bg_color;

    let mut x = rect.left();
    while x < rect.right() {
        let column = Rect::from_min_size(pos2(x, rect.top()), vec2(40.0, rect.height()));
        ui.painter().rect_filled(column, 0.0, fill_color);
        x += 91.0;
    }

    let mut y = rect.top();
    while y < rect.bottom() {
        let row = Rect::from_min_size(pos2(rect.left(), y), vec2(rect.width(), 20.0));
        ui.painter().rect_filled(row, 0.0, fill_color);
        y += 43.0;
    }
}

/// A scroll area with some portion of its left and/or top side "stuck".
///
/// This produces four quadrants:
///
/// ```text
///               <-------LEFT-------> <---------RIGHT---------->
///
///              ------------------------------------------------
///          ^   |                    |   <----------------->   |
///    TOP   |   |       Fixed        |      Horizontally       |
///          V   |                    |       scrollable        |
///              |--------------------|-------------------------|
///          ^   | ^                  |           ^             |
///  BOTTOM  |   | |   Vertically     | <-  Fully scrollable -> |
///          V   | v   scrollable     |           v             |
///              |____________________|_________________________|
/// ```
struct SplitScroll {
    scroll_enabled: Vec2b,

    /// Width of the fixed left side, and height of the fixed top.
    fixed_size: Vec2,

    /// Size of the small container of the right bottom scrollable region.
    scroll_outer_size: Vec2,

    /// Size of the large contents of the right bottom region, ignoring the left/top fixed regions.
    scroll_content_size: Vec2,
}

impl SplitScroll {
    fn show(self, ui: &mut Ui, delegate: &mut dyn PartialScrollDelegate) {
        let Self {
            scroll_enabled,
            fixed_size,
            scroll_outer_size,
            scroll_content_size,
        } = self;

        ui.scope(|ui| {
            ui.visuals_mut().clip_rect_margin = 0.0; // Everything else looks awful

            let mut rect = ui.cursor();
            rect.max = rect.min + fixed_size + scroll_outer_size;

            // TODO: scroll-wheel input when hovering top/left regions
            // TODO: drag-to-scroll on top/left regions
            // TODO: pass on visible region to delegate

            let bottom_right_rect = Rect::from_min_max(rect.min + fixed_size, rect.max);
            let scroll_offset = ui
                .scope_builder(UiBuilder::new().max_rect(bottom_right_rect), |ui| {
                    egui::ScrollArea::new(scroll_enabled)
                        .show_viewport(ui, |ui, viewport| {
                            ui.set_min_size(scroll_content_size);
                            delegate.right_bottom_ui(ui);
                        })
                        .state
                        .offset
                })
                .inner;

            {
                // Fixed
                let left_top_rect = rect
                    .with_max_x(rect.left() + fixed_size.x)
                    .with_max_y(rect.top() + fixed_size.y);
                let mut left_top_ui = ui.new_child(UiBuilder::new().max_rect(left_top_rect));
                delegate.left_top_ui(&mut left_top_ui);
            }

            {
                // Horizontally scrollable
                let right_top_outer_rect = rect
                    .with_min_x(rect.left() + fixed_size.x)
                    .with_max_y(rect.top() + fixed_size.y);
                let right_top_content_rect = Rect::from_min_size(
                    pos2(right_top_outer_rect.min.x - scroll_offset.x, rect.min.y),
                    vec2(scroll_content_size.x, fixed_size.y),
                );
                let mut right_top_ui =
                    ui.new_child(UiBuilder::new().max_rect(right_top_content_rect));
                right_top_ui.set_clip_rect(right_top_outer_rect);
                delegate.right_top_ui(&mut right_top_ui);
            }

            {
                // Vertically scrollable
                let left_bottom_outer_rect = rect
                    .with_max_x(rect.left() + fixed_size.x)
                    .with_min_y(rect.top() + fixed_size.y);
                let left_bottom_content_rect = Rect::from_min_size(
                    pos2(rect.min.x, left_bottom_outer_rect.min.y - scroll_offset.y),
                    vec2(fixed_size.x, scroll_content_size.y),
                );
                let mut left_bottom_ui =
                    ui.new_child(UiBuilder::new().max_rect(left_bottom_content_rect));
                left_bottom_ui.set_clip_rect(left_bottom_outer_rect);
                delegate.left_bottom_ui(&mut left_bottom_ui);
            }
        });
    }
}

trait PartialScrollDelegate {
    /// The fixed portion of the top left corner.
    fn left_top_ui(&mut self, ui: &mut Ui);

    /// The horizontally scrollable portion.
    fn right_top_ui(&mut self, ui: &mut Ui);

    /// The vertically scrollable portion.
    fn left_bottom_ui(&mut self, ui: &mut Ui);

    /// The fully scrollable portion.
    fn right_bottom_ui(&mut self, ui: &mut Ui);
}
