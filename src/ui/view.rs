use iced::widget::{column, container, mouse_area, row, text, text_input};
use iced::{Border, Color, Element, Length, Theme, widget};

use crate::compositor::{FullscreenStatus, Process};
use crate::config_management::parse_colour;

use super::update::Message;

use super::{AppState, TEXT_INPUT_ID};

impl AppState {
    fn client_item<'a>(
        &'a self,
        idx: usize,
        client: &Process,
        name: &'a str,
    ) -> Element<'a, Message> {
        let is_selected = idx == self.selected_idx;
        let title = name;
        let workspace_id = client.workspace;
        let status_col = match client.fullscreen {
            FullscreenStatus::Fullscreen => parse_colour(&self.config.colours.status.fullscreen),
            FullscreenStatus::Maximised => parse_colour(&self.config.colours.status.maximized),
            FullscreenStatus::None => {
                if client.floating {
                    parse_colour(&self.config.colours.status.floating)
                } else {
                    parse_colour(&self.config.colours.status.tiled)
                }
            }
        };
        let status = match client.fullscreen {
            FullscreenStatus::Fullscreen => "Fullscreen",
            FullscreenStatus::Maximised => "Maximised",
            FullscreenStatus::None => {
                if client.floating {
                    "Float"
                } else {
                    "Tiled"
                }
            }
        };

        // These are split into parts so they can have different colours.
        // implementation for ALL of these colours will be added sometime later.
        // Currently only supports status colours
        let title_part = text(title);
        let workspace_part = if workspace_id > 50 {
            // atleast for me, my special workspace (in a
            // scratch pad) is on workspace -98 -
            // assuming it uses the same logic, any
            // special workspace is a negative number
            // Since this is now converted into an unsigned
            // integer, this will now be above 50
            // Surelyt no one has mroe than 50 worskpaces right
            text("@Workspace: Special Workspace")
        } else {
            text(format!("@Workspace: {workspace_id}"))
        };
        let status_part = text(format!("[{status}]")).style(move |_| text::Style {
            color: Some(status_col),
        });

        // brings all together
        let item_content: widget::Row<'_, _, _, _> =
            row!(title_part, workspace_part, status_part).spacing(self.config.layout.spacing);

        let styled = if is_selected {
            container(item_content)
                .width(Length::Fill)
                .style(|theme: &Theme| container::Style {
                    // Using Iced's built-in palette for primary/background
                    background: Some(theme.palette().primary.into()),
                    text_color: Some(theme.palette().background),
                    border: Border {
                        radius: self.config.layout.border_radius.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                // Honestly looks better like this. Tried making the edges wrap around the text but i couldn't
                // figure it out so this is the best way to make it look "pretty"
                // So example:
                // **When it is not selected:**
                //    item blah blah blah
                // **When it is selected:**
                // -----------------------
                // | item blah blah blah |
                // -----------------------
                // Not the greatest, but it looks nice.
                .padding(self.config.layout.padding)
        } else {
            container(item_content)
                .width(Length::Fill)
                .style(|theme: &Theme| container::Style {
                    background: Some(Color::TRANSPARENT.into()),
                    text_color: Some(theme.palette().text),
                    border: Border {
                        radius: self.config.layout.border_radius.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .padding([self.config.layout.padding, self.config.layout.margin])
        }
        .width(Length::Shrink);

        mouse_area(styled)
            .on_press(Message::SelectAndFocus(idx))
            .on_right_press(Message::SelectAndClose(idx))
            .on_enter(Message::HoverWindow(idx))
            .interaction(iced::mouse::Interaction::Pointer)
            .into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let items: Vec<_> = self
            .clients_to_display
            .iter()
            .enumerate()
            .map(|(idx, (client, name))| self.client_item(idx, client, name))
            .collect();
        let search_bar_widget = Element::from(
            text_input("Search", &self.query)
                .id(TEXT_INPUT_ID.clone())
                .style(|_, _| text_input::Style {
                    background: parse_colour(&self.config.colours.search_background).into(),
                    border: Border {
                        color: parse_colour(&self.config.colours.search_border_col),
                        radius: self.config.layout.border_radius.into(),
                        width: 1.0,
                    },
                    selection: parse_colour(&self.config.colours.selected_text),
                    icon: parse_colour(&self.config.colours.selected_background),
                    placeholder: parse_colour(&self.config.colours.text),
                    value: parse_colour(&self.config.colours.selected_text),
                })
                .on_input(Message::UpdateInput)
                .on_submit(Message::ClientSelected)
                .padding(self.config.layout.padding)
                .size(self.config.font.size),
        );

        let scrollable_list: Element<'_, Message> = widget::Scrollable::new(
            column(items).spacing(self.config.layout.spacing), // Put all client elements in a column
        )
        .width(Length::Fill)
        .height(Length::Fill) // The scrollable part should fill available vertical space
        .id(self.scroll_id.clone())
        .into();

        let root_layout = column![search_bar_widget, scrollable_list,]
            .spacing(self.config.layout.spacing)
            .width(Length::Fill)
            .height(Length::Fill);

        container(root_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(self.config.layout.margin) // Outer margin
            .style(|_theme| container::Style {
                background: Some(parse_colour(&self.config.colours.background).into()),
                border: Border {
                    color: parse_colour(&self.config.colours.border_col),
                    width: 2.0, // Adjust as needed
                    radius: self.config.layout.border_radius.into(),
                },
                ..Default::default()
            })
            .into()
    }
}
