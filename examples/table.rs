use iced::{
    Alignment::Center,
    Length::Fill,
    Subscription, Task,
    widget::{container, scrollable, text},
};
use iced_table_fluid::table;

#[derive(Clone)]
pub enum Message {
    WindowResized(iced::Size),
}

#[derive(Clone)]
pub struct Item {
    pub column_1: f64,
    pub column_2: f64,
    pub column_3: f64,
    pub column_4: f64,
    pub column_5: f64,
    pub column_6: f64,
    pub column_7: f64,
    pub column_8: f64,
    pub column_9: f64,
    pub column_10: f64,
}

pub struct State {
    items: Vec<Item>,
    window_size: iced::Size,
}

impl Default for State {
    fn default() -> Self {
        let items = (1..40)
            .map(|_| Item {
                column_1: (10.0_f64).powf(1.0),
                column_2: (10.0_f64).powf(2.0),
                column_3: (10.0_f64).powf(3.0),
                column_4: (10.0_f64).powf(4.0),
                column_5: (10.0_f64).powf(5.0),
                column_6: (10.0_f64).powf(6.0),
                column_7: (10.0_f64).powf(7.0),
                column_8: (10.0_f64).powf(8.0),
                column_9: (10.0_f64).powf(12.0),
                column_10: (10.0_f64).powf(10.0),
            })
            .collect::<Vec<_>>();

        Self {
            items,
            window_size: iced::Size::default(),
        }
    }
}

pub fn main() -> iced::Result {
    iced::application(State::default, update, view)
        .subscription(subscription)
        .run()
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::WindowResized(size) => state.window_size = size,
    }
    Task::none()
}

fn view(state: &State) -> iced::Element<'_, Message> {
    let bold = |value| text(value);
    let cell = |value: f64| text(value.to_string());
    scrollable(
        container(
            table::table(
                [
                    table::column(bold("Column 1"), |value: Item| cell(value.column_1))
                        .align_y(Center),
                    table::column(bold("Column 2"), |value: Item| cell(value.column_2))
                        .align_y(Center),
                    table::column(bold("Column 3"), |value: Item| cell(value.column_3))
                        .align_y(Center),
                    table::column(bold("Column 4"), |value: Item| cell(value.column_4))
                        .align_y(Center),
                    table::column(bold("Column 5"), |value: Item| cell(value.column_5))
                        .align_y(Center),
                    table::column(bold("Column 6"), |value: Item| cell(value.column_6))
                        .align_y(Center),
                    table::column(bold("Column 7"), |value: Item| cell(value.column_7))
                        .align_y(Center),
                    table::column(bold("Column 8"), |value: Item| cell(value.column_8))
                        .align_y(Center),
                    table::column(bold("Column 9"), |value: Item| cell(value.column_9))
                        .align_y(Center),
                    table::column(bold("Column 10"), |value: Item| cell(value.column_10))
                        .align_y(Center),
                ],
                state.items.clone(),
            )
            .max_width(state.window_size.width),
        )
        .center_x(Fill)
        .style(container::bordered_box),
    )
    .direction(scrollable::Direction::Both {
        vertical: scrollable::Scrollbar::default(),
        horizontal: scrollable::Scrollbar::default(),
    })
    .into()
}

fn subscription(_: &State) -> Subscription<Message> {
    iced::window::resize_events().map(|(_id, size)| Message::WindowResized(size))
}
