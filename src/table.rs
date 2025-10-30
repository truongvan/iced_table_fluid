//! Display tables.
use iced::advanced::widget::{Operation, tree};
use iced::advanced::{self, Layout, Renderer as R, Widget, layout, overlay, renderer};
use iced::alignment;
use iced::mouse;
use iced::{Alignment, Background, Element, Length, Pixels, Rectangle, Size};

/// Creates a new [`Table`] with the given columns and rows.
///
/// Columns can be created using the [`column()`] function, while rows can be any
/// iterator over some data type `T`.
pub fn table<'a, 'b, T, Message, Theme, Renderer>(
    columns: impl IntoIterator<Item = Column<'a, 'b, T, Message, Theme, Renderer>>,
    rows: impl IntoIterator<Item = T>,
) -> Table<'a, Message, Theme, Renderer>
where
    T: Clone,
    Theme: Catalog,
    Renderer: R,
{
    Table::new(columns, rows)
}

/// Creates a new [`Column`] with the given header and view function.
///
/// The view function will be called for each row in a [`Table`] and it must
/// produce the resulting contents of a cell.
pub fn column<'a, 'b, T, E, Message, Theme, Renderer>(
    header: impl Into<Element<'a, Message, Theme, Renderer>>,
    view: impl Fn(T) -> E + 'b,
) -> Column<'a, 'b, T, Message, Theme, Renderer>
where
    T: 'a,
    E: Into<Element<'a, Message, Theme, Renderer>>,
{
    Column {
        header: header.into(),
        view: Box::new(move |data| view(data).into()),
        width: Length::Shrink,
        align_x: alignment::Horizontal::Left,
        align_y: alignment::Vertical::Top,
    }
}

/// A grid-like visual representation of data distributed in columns and rows.
pub struct Table<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
{
    columns: Vec<Column_>,
    cells: Vec<Element<'a, Message, Theme, Renderer>>,
    width: Length,
    height: Length,
    max_width: Length,
    padding_x: f32,
    padding_y: f32,
    separator_x: f32,
    separator_y: f32,
    class: Theme::Class<'a>,
}

struct Column_ {
    width: Length,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
}

impl<'a, Message, Theme, Renderer> Table<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: R,
{
    /// Creates a new [`Table`] with the given columns and rows.
    ///
    /// Columns can be created using the [`column()`] function, while rows can be any
    /// iterator over some data type `T`.
    pub fn new<'b, T>(
        columns: impl IntoIterator<Item = Column<'a, 'b, T, Message, Theme, Renderer>>,
        rows: impl IntoIterator<Item = T>,
    ) -> Self
    where
        T: Clone,
    {
        let columns = columns.into_iter();
        let rows = rows.into_iter();

        let mut width = Length::Shrink;
        let mut height = Length::Shrink;

        let mut cells = Vec::with_capacity(columns.size_hint().0 * (1 + rows.size_hint().0));

        let (mut columns, views): (Vec<_>, Vec<_>) = columns
            .map(|column| {
                width = width.enclose(column.width);

                cells.push(column.header);

                (
                    Column_ {
                        width: column.width,
                        align_x: column.align_x,
                        align_y: column.align_y,
                    },
                    column.view,
                )
            })
            .collect();

        for row in rows {
            for view in &views {
                let cell = view(row.clone());
                let size_hint = cell.as_widget().size_hint();

                height = height.enclose(size_hint.height);

                cells.push(cell);
            }
        }

        if width == Length::Shrink
            && let Some(first) = columns.first_mut()
        {
            first.width = Length::Fill;
        }

        let max_width = Length::Fill;

        Self {
            columns,
            cells,
            width,
            max_width,
            height,
            padding_x: 10.0,
            padding_y: 5.0,
            separator_x: 1.0,
            separator_y: 1.0,
            class: Theme::default(),
        }
    }

    /// Sets the width of the [`Table`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the max_width of the [`Table`].
    pub fn max_width(mut self, width: impl Into<Length>) -> Self {
        self.max_width = width.into();
        self
    }

    /// Sets the padding of the cells of the [`Table`].
    pub fn padding(self, padding: impl Into<Pixels>) -> Self {
        let padding = padding.into();

        self.padding_x(padding).padding_y(padding)
    }

    /// Sets the horizontal padding of the cells of the [`Table`].
    pub fn padding_x(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding_x = padding.into().0;
        self
    }

    /// Sets the vertical padding of the cells of the [`Table`].
    pub fn padding_y(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding_y = padding.into().0;
        self
    }

    /// Sets the thickness of the line separator between the cells of the [`Table`].
    pub fn separator(self, separator: impl Into<Pixels>) -> Self {
        let separator = separator.into();

        self.separator_x(separator).separator_y(separator)
    }

    /// Sets the thickness of the horizontal line separator between the cells of the [`Table`].
    pub fn separator_x(mut self, separator: impl Into<Pixels>) -> Self {
        self.separator_x = separator.into().0;
        self
    }

    /// Sets the thickness of the vertical line separator between the cells of the [`Table`].
    pub fn separator_y(mut self, separator: impl Into<Pixels>) -> Self {
        self.separator_y = separator.into().0;
        self
    }
}

struct Metrics {
    columns: Vec<f32>,
    rows: Vec<f32>,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Table<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: R,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Metrics>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Metrics {
            columns: Vec::new(),
            rows: Vec::new(),
        })
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.cells
            .iter()
            .map(|cell| tree::Tree::new(cell.as_widget()))
            .collect()
    }

    fn diff(&self, state: &mut tree::Tree) {
        state.diff_children(&self.cells);
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let metrics = tree.state.downcast_mut::<Metrics>();
        let columns = self.columns.len();
        let rows = self.cells.len() / columns;

        let limits = limits.width(self.width).height(self.height);
        let available = limits.max();
        let max_limits = limits.width(self.max_width).height(self.height).max();

        let mut cells = Vec::with_capacity(self.cells.len());
        cells.resize(self.cells.len(), layout::Node::default());

        metrics.columns = vec![0.0; columns];
        metrics.rows = vec![0.0; rows];

        // We keep row height logic (factors & distribution) intact
        let mut total_row_factors = 0;
        let mut total_fluid_height = 0.0;
        let mut row_factor = 0;

        // spacing_x includes per-column left+right padding plus the separator
        let spacing_x = self.padding_x * 2.0 + self.separator_x;
        let spacing_y = self.padding_y * 2.0 + self.separator_y;

        // ---------- FIRST PASS ----------
        // Ignore declared column widths: treat as Shrink to measure intrinsic widths per column.
        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, (cell, state)) in self.cells.iter_mut().zip(&mut tree.children).enumerate() {
            let row = i / columns;
            let column = i % columns;

            if column == 0 {
                x = self.padding_x;

                if row > 0 {
                    y += metrics.rows[row - 1] + spacing_y;

                    if row_factor != 0 {
                        total_fluid_height += metrics.rows[row - 1];
                        total_row_factors += row_factor;
                        row_factor = 0;
                    }
                }
            }

            let size_req = cell.as_widget().size();
            let height_factor = size_req.height.fill_factor();
            row_factor = row_factor.max(height_factor);

            // Layout with width forced to Shrink, so we can measure intrinsic content width.
            let max = Size::new(available.width - x, available.height - y);
            let pass1_limits = layout::Limits::new(Size::ZERO, max).width(Length::Shrink);

            let layout = cell.as_widget_mut().layout(state, renderer, &pass1_limits);
            let sz = pass1_limits.resolve(Length::Shrink, Length::Shrink, layout.size());

            // Per-column intrinsic width (content), accumulated as max
            metrics.columns[column] = metrics.columns[column].max(sz.width);

            // Row height metrics only for non-fluid rows (existing behavior preserved)
            if height_factor == 0 && !size_req.height.is_fill() {
                metrics.rows[row] = metrics.rows[row].max(sz.height);
            }

            // Store node for now; it will be re-laid out in pass 2
            cells[i] = layout;

            x += sz.width + spacing_x;
        }

        // Account for last row's factors
        if row_factor != 0 && rows > 0 {
            total_fluid_height += metrics.rows[rows - 1];
            total_row_factors += row_factor;
        }

        // ---------- WIDTH SHARING ----------
        // Compute remaining parent width and distribute evenly across columns,
        // then lock columns to Fixed(intrinsic + share).
        let content_available = (available.width.min(max_limits.width)
            - self.padding_x * 2.0
            - spacing_x * columns.saturating_sub(1) as f32)
            .max(0.0);

        let content_intrinsic: f32 = metrics.columns.iter().copied().sum::<f32>();
        let remaining = (content_available - content_intrinsic).max(0.0);
        let share = if columns == 0 {
            0.0
        } else {
            remaining / columns as f32
        };

        // let mut fixed_widths = vec![0.0; columns];
        metrics.columns = metrics.columns.iter().map(|v| v + share).collect();
        let fixed_widths = metrics.columns.clone();

        // ---------- SECOND PASS ----------
        // Height logic (row factors & distribution) is unchanged.
        let left_height = available.height - total_fluid_height;
        let height_unit = if total_row_factors == 0 {
            0.0
        } else {
            (left_height - spacing_y * rows.saturating_sub(1) as f32 - self.padding_y * 2.0)
                / total_row_factors as f32
        };

        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, (cell, state)) in self.cells.iter_mut().zip(&mut tree.children).enumerate() {
            let row = i / columns;
            let column = i % columns;

            if column == 0 {
                x = self.padding_x;

                if row > 0 {
                    y += metrics.rows[row - 1] + spacing_y;
                }
            }

            let size_req = cell.as_widget().size();
            let height_factor = size_req.height.fill_factor();

            let max_height = if height_factor == 0 {
                if size_req.height.is_fill() {
                    metrics.rows[row]
                } else {
                    (available.height - y).max(0.0)
                }
            } else {
                height_unit * height_factor as f32
            };

            // Force column width to Fixed(intrinsic + share)
            let fixed = Length::Fixed(fixed_widths[column]);

            let pass2_limits =
                layout::Limits::new(Size::ZERO, Size::new(available.width - x, max_height))
                    .width(fixed);

            let layout = cell.as_widget_mut().layout(state, renderer, &pass2_limits);
            let sz = pass2_limits.resolve(fixed, Length::Shrink, layout.size());

            // Row metric grows as usual
            metrics.rows[row] = metrics.rows[row].max(sz.height);

            cells[i] = layout;
            x += fixed_widths[column] + spacing_x;
        }

        // ---------- THIRD PASS (position) ----------
        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, cell) in cells.iter_mut().enumerate() {
            let row = i / columns;
            let column = i % columns;

            if column == 0 {
                x = self.padding_x;

                if row > 0 {
                    y += metrics.rows[row - 1] + spacing_y;
                }
            }

            let Column_ {
                align_x, align_y, ..
            } = &self.columns[column];

            cell.move_to_mut((x, y));
            cell.align_mut(
                Alignment::from(*align_x),
                Alignment::from(*align_y),
                Size::new(metrics.columns[column], metrics.rows[row]),
            );

            x += metrics.columns[column] + spacing_x;
        }

        // Intrinsic table size
        let intrinsic = limits.resolve(
            self.width,
            self.height,
            Size::new(
                // left pad + sum(fixed) + separators + right pad
                x - spacing_x + self.padding_x,
                // top pad + rows + inter-row spacing + bottom pad
                self.padding_y * 2.0
                    + metrics.rows.iter().sum::<f32>()
                    + spacing_y * rows.saturating_sub(1) as f32
                    - self.separator_y, // remove the last added separator_y
            ),
        );

        layout::Node::with_children(intrinsic, cells)
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((cell, state), layout) in self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            cell.as_widget_mut().update(
                state, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((cell, state), layout) in self.cells.iter().zip(&tree.children).zip(layout.children())
        {
            cell.as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }

        let bounds = layout.bounds();
        let metrics = tree.state.downcast_ref::<Metrics>();
        let style = theme.style(&self.class);

        if self.separator_x > 0.0 {
            let mut x = self.padding_x;

            for width in &metrics.columns[..metrics.columns.len().saturating_sub(1)] {
                x += width + self.padding_x;

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + x,
                            y: bounds.y,
                            width: self.separator_x,
                            height: bounds.height,
                        },
                        snap: true,
                        ..renderer::Quad::default()
                    },
                    style.separator_x,
                );

                x += self.separator_x + self.padding_x;
            }
        }

        if self.separator_y > 0.0 {
            let mut y = self.padding_y;

            for height in &metrics.rows[..metrics.rows.len().saturating_sub(1)] {
                y += height + self.padding_y;

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y: bounds.y + y,
                            width: bounds.width,
                            height: self.separator_y,
                        },
                        snap: true,
                        ..renderer::Quad::default()
                    },
                    style.separator_y,
                );

                y += self.separator_y + self.padding_y;
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &tree::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.cells
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((cell, state), layout)| {
                cell.as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn operate(
        &mut self,
        tree: &mut tree::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        for ((cell, state), layout) in self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            cell.as_widget_mut()
                .operate(state, layout, renderer, operation);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut tree::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.cells,
            state,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Table<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: R + 'a,
{
    fn from(table: Table<'a, Message, Theme, Renderer>) -> Self {
        Element::new(table)
    }
}

/// A vertical visualization of some data with a header.
pub struct Column<'a, 'b, T, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    header: Element<'a, Message, Theme, Renderer>,
    view: Box<dyn Fn(T) -> Element<'a, Message, Theme, Renderer> + 'b>,
    width: Length,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
}

impl<'a, 'b, T, Message, Theme, Renderer> Column<'a, 'b, T, Message, Theme, Renderer> {
    /// Sets the width of the [`Column`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the alignment for the horizontal axis of the [`Column`].
    pub fn align_x(mut self, alignment: impl Into<alignment::Horizontal>) -> Self {
        self.align_x = alignment.into();
        self
    }

    /// Sets the alignment for the vertical axis of the [`Column`].
    pub fn align_y(mut self, alignment: impl Into<alignment::Vertical>) -> Self {
        self.align_y = alignment.into();
        self
    }
}

/// The appearance of a [`Table`].
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The background color of the horizontal line separator between cells.
    pub separator_x: Background,
    /// The background color of the vertical line separator between cells.
    pub separator_y: Background,
}

/// The theme catalog of a [`Table`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Table`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl<Theme> From<Style> for StyleFn<'_, Theme> {
    fn from(style: Style) -> Self {
        Box::new(move |_theme| style)
    }
}

impl Catalog for iced::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The default style of a [`Table`].
pub fn default(theme: &iced::Theme) -> Style {
    let palette = theme.extended_palette();
    let separator = palette.background.strong.color.into();

    Style {
        separator_x: separator,
        separator_y: separator,
    }
}
