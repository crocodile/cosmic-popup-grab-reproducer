// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    Element,
    app::{Core, Task},
    iced::{
        platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup},
        widget::container,
        window,
    },
    widget::mouse_area,
};

const APP_ID: &str = "io.github.crocodile.CosmicPopupGrabReproducer";

struct PopupGrabReproducer {
    core: Core,
    popup: Option<window::Id>,
}

#[derive(Clone, Debug)]
enum Message {
    Noop,
    TogglePopup,
    PopupClosed(window::Id),
    #[cfg(feature = "tooltip")]
    Surface(cosmic::surface::Action),
}

impl cosmic::Application for PopupGrabReproducer {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        (Self { core, popup: None }, Task::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Noop => {}
            Message::TogglePopup => {
                return if let Some(popup) = self.popup.take() {
                    eprintln!("closing popup {popup:?}");
                    destroy_popup(popup)
                } else {
                    let popup = window::Id::unique();
                    self.popup = Some(popup);
                    eprintln!("opening popup {popup:?}");

                    let settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().expect("applet main window"),
                        popup,
                        Some((1, 1)),
                        None,
                        None,
                    );

                    get_popup(settings)
                };
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            #[cfg(feature = "tooltip")]
            Message::Surface(action) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(action),
                ));
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let button = self.core.applet.text_button("POP", Message::Noop);
        #[cfg(feature = "tooltip")]
        let button = self.core.applet.applet_tooltip(
            button,
            "Popup grab reproducer",
            false,
            Message::Surface,
            None,
        );

        mouse_area(button)
            .on_right_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Self::Message> {
        let content = self.core.applet.text("Hello from a popup");

        self.core
            .applet
            .popup_container(container(content).padding(16))
            .into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<PopupGrabReproducer>(())
}
