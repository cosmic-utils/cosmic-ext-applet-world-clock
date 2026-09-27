use std::{sync::LazyLock, time::Duration};

use cosmic::cosmic_config::CosmicConfigEntry;
use jiff::{Zoned, tz::TimeZone};
use tracing::{debug, trace};

use crate::{
    config::{APP_ID, ClockConfig, Flags, WorldClockConfig},
    fl,
};

/// All IANA timezone names, for the picker dropdown.
static TIMEZONES: LazyLock<Vec<String>> = LazyLock::new(|| {
    jiff::tz::db()
        .available()
        .map(|name| name.as_str().to_string())
        .collect()
});

pub(crate) fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<WorldClock>(Flags::new())
}

struct WorldClock {
    core: cosmic::app::Core,
    size: cosmic::iced::Size,
    popup: Option<cosmic::iced::window::Id>,
    config: WorldClockConfig,
    config_handler: Option<cosmic::cosmic_config::Config>,
    now: Zoned,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Tick,
    Size(cosmic::iced::Size),
    ToggleWindow,
    PopupClosed(cosmic::iced::window::Id),
    TimezoneEdited(usize, String),
    LabelEdited(usize, String),
    RemoveClock(usize),
    AddClock,
    ToggleSeconds(usize, bool),
    ToggleMilitary(usize, bool),
    ToggleDate(usize, bool),
    ToggleWeekday(usize, bool),
}

impl WorldClock {
    fn clock_label(&self, clock: &ClockConfig) -> String {
        // None = untouched, fall back to the timezone name.
        // Some("") = explicitly cleared by the user; stays empty.
        clock
            .label
            .clone()
            .unwrap_or_else(|| clock.timezone.clone())
    }

    /// Format the current time in `clock.timezone`, optionally prefixing the
    /// date (and weekday) the way the system time applet does.
    fn format_time(&self, clock: &ClockConfig) -> String {
        format_time(
            &self.now,
            &clock.timezone,
            clock.military_time,
            clock.show_seconds,
            clock.show_weekday,
            clock.show_date,
        )
    }

    fn persist(&self) {
        if let Some(handler) = &self.config_handler
            && let Err(error) = self.config.write_entry(handler)
        {
            tracing::error!("failed to persist config: {error}");
        }
    }
}

/// A labelled toggle row (label on the left, switch on the right), used for
/// the per-clock format options in the popup.
fn toggle_row(
    label: String,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'static,
) -> cosmic::Element<'static, Message> {
    cosmic::widget::row::with_capacity(3)
        .push(cosmic::widget::text(label))
        .push(cosmic::widget::space::horizontal())
        .push(cosmic::widget::toggler(value).on_toggle(on_toggle))
        .into()
}

/// Format `now` in `timezone` according to the 12/24h, seconds, date, and
/// weekday settings. The date (and weekday) prefix matches the system time
/// applet's `MDT`/`MDET` medium style (`Sep 27, 12:18 PM` / `Sun, Sep 27, 12:18 PM`).
fn format_time(
    now: &Zoned,
    timezone: &str,
    military: bool,
    seconds: bool,
    date: bool,
    weekday: bool,
) -> String {
    let zoned = match TimeZone::get(timezone) {
        Ok(tz) => now.with_time_zone(tz),
        Err(_) => now.with_time_zone(TimeZone::UTC),
    };

    let time = match (military, seconds) {
        (true, true) => "%H:%M:%S",
        (true, false) => "%H:%M",
        (false, true) => "%-I:%M:%S %p",
        (false, false) => "%-I:%M %p",
    };
    // Weekday only shows with the date, matching the system time applet.
    let date = match (weekday, date) {
        (true, true) => "%a, %b %-d, ",
        (false, true) => "%b %-d, ",
        _ => "",
    };

    zoned.strftime(&format!("{date}{time}")).to_string()
}

impl cosmic::Application for WorldClock {
    type Flags = Flags;
    type Message = Message;
    type Executor = cosmic::SingleThreadExecutor;

    const APP_ID: &'static str = APP_ID;

    fn init(
        core: cosmic::app::Core,
        flags: Self::Flags,
    ) -> (Self, cosmic::app::Task<Self::Message>) {
        (
            Self {
                core,
                popup: None,
                config: flags.config,
                config_handler: flags.config_handler,
                now: Zoned::now(),
                size: cosmic::iced::Size {
                    width: 10.,
                    height: 10.,
                },
            },
            cosmic::task::none(),
        )
    }

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        cosmic::iced::Subscription::batch([
            cosmic::iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick),
            cosmic::iced::event::listen_with(|event, _status, id| {
                if let cosmic::iced::Event::Window(
                    cosmic::iced::window::Event::Resized(size)
                    | cosmic::iced::window::Event::Opened { position: _, size },
                ) = event
                    && id == cosmic::iced::window::Id::RESERVED
                {
                    Some(Message::Size(size))
                } else {
                    None
                }
            }),
        ])
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    fn on_close_requested(&self, id: cosmic::iced::window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Message) -> cosmic::app::Task<Self::Message> {
        match message {
            Message::Tick => trace!(?message),
            _ => debug!(?message),
        }

        match message {
            Message::Tick => {
                self.now = Zoned::now();
            }
            Message::Size(size) => {
                self.size = size;
            }
            Message::ToggleWindow => {
                if let Some(id) = self.popup.take() {
                    return cosmic::iced::platform_specific::shell::commands::popup::destroy_popup(
                        id,
                    );
                }

                let new_id = cosmic::iced::window::Id::unique();
                self.popup.replace(new_id);

                let mut popup_settings = self.core.applet.get_popup_settings(
                    self.core
                        .main_window_id()
                        .expect("applet should always have a main window"),
                    new_id,
                    None,
                    None,
                    None,
                );
                popup_settings.positioner.anchor_rect = cosmic::iced::Rectangle::<i32> {
                    x: 0,
                    y: 0,
                    width: self.size.width as i32,
                    height: self.size.height as i32,
                };

                // Let the compositor size the popup from the content (like
                // the time applet does), otherwise it keeps the default
                // panel-derived size and shows empty space top and bottom.
                popup_settings.positioner.size = None;

                return cosmic::iced::platform_specific::shell::commands::popup::get_popup(
                    popup_settings,
                );
            }
            Message::PopupClosed(id) => {
                self.popup.take_if(|stored_id| stored_id == &id);
            }
            Message::TimezoneEdited(index, timezone) => {
                if let Some(clock) = self.config.clocks.get_mut(index) {
                    clock.timezone = timezone;
                    self.persist();
                }
            }
            Message::LabelEdited(index, value) => {
                if let Some(clock) = self.config.clocks.get_mut(index) {
                    clock.label = Some(value);
                    self.persist();
                }
            }
            Message::RemoveClock(index) => {
                if index < self.config.clocks.len() {
                    self.config.clocks.remove(index);
                    self.persist();
                }
            }
            Message::AddClock => {
                self.config.clocks.push(ClockConfig {
                    timezone: String::new(),
                    ..ClockConfig::default()
                });
                self.persist();
            }
            Message::ToggleSeconds(index, value) => {
                if let Some(clock) = self.config.clocks.get_mut(index) {
                    clock.show_seconds = value;
                    self.persist();
                }
            }
            Message::ToggleMilitary(index, value) => {
                if let Some(clock) = self.config.clocks.get_mut(index) {
                    clock.military_time = value;
                    self.persist();
                }
            }
            Message::ToggleWeekday(index, value) => {
                if let Some(clock) = self.config.clocks.get_mut(index) {
                    clock.show_weekday = value;
                    self.persist();
                }
            }
            Message::ToggleDate(index, value) => {
                if let Some(clock) = self.config.clocks.get_mut(index) {
                    clock.show_date = value;
                    self.persist();
                }
            }
        }

        cosmic::task::none()
    }

    fn view(&self) -> cosmic::Element<'_, Message> {
        let mut row = cosmic::widget::row::with_capacity(self.config.clocks.len())
            .align_y(cosmic::iced::Alignment::Center)
            .spacing(8);

        for clock in &self.config.clocks {
            let time = self.format_time(clock);
            let label = self.clock_label(clock);

            // Each clock is a single horizontal line, so clocks sit side by side.
            let mut clock_row = cosmic::widget::row::with_capacity(3)
                .align_y(cosmic::iced::Alignment::Center)
                .spacing(6);

            if !label.is_empty() {
                clock_row = clock_row.push(self.core.applet.text(label));
            }
            clock_row = clock_row.push(self.core.applet.text(time));

            row = row.push(clock_row);
        }

        let button = cosmic::widget::button::custom(row)
            .class(cosmic::theme::Button::AppletIcon)
            .on_press_down(Message::ToggleWindow);

        cosmic::widget::autosize::autosize(button, cosmic::widget::Id::unique()).into()
    }

    fn view_window(&self, _id: cosmic::iced::window::Id) -> cosmic::Element<'_, Message> {
        let mut list = cosmic::widget::column::with_capacity(self.config.clocks.len() + 4)
            .spacing(8)
            .padding([0, 16]);

        for (index, clock) in self.config.clocks.iter().enumerate() {
            let query = clock.timezone.clone();
            let query_lower = query.to_lowercase();
            let matches = if TIMEZONES.iter().any(|tz| tz == &query) {
                Vec::new()
            } else {
                TIMEZONES
                    .iter()
                    .filter(|tz| tz.to_lowercase().contains(&query_lower))
                    .take(8)
                    .collect()
            };

            let timezone_input =
                cosmic::widget::text_input::text_input(fl!("search-timezone"), &clock.timezone)
                    .on_input(move |value| Message::TimezoneEdited(index, value));

            let placeholder = fl!("label-placeholder");
            let label_value = self.clock_label(clock);

            let label_input =
                cosmic::widget::text_input::text_input(placeholder, label_value.clone())
                    .on_input(move |value| Message::LabelEdited(index, value));

            let remove = cosmic::widget::button::icon(cosmic::widget::icon::from_name(
                "edit-delete-symbolic",
            ))
            .on_press(Message::RemoveClock(index))
            .class(cosmic::theme::Button::Destructive);

            let top_row = cosmic::widget::row::with_capacity(3)
                .push(timezone_input)
                .push(label_input)
                .push(remove)
                .spacing(8);

            // Each clock occupies a full-width column so long timezone names aren't cut off.
            let mut clock_box = cosmic::widget::column::with_capacity(2)
                .push(top_row)
                .spacing(4);

            if !matches.is_empty() {
                let mut match_list =
                    cosmic::widget::column::with_capacity(matches.len()).spacing(2);

                for tz in matches {
                    match_list = match_list.push(
                        cosmic::widget::button::text(tz.clone())
                            .on_press(Message::TimezoneEdited(index, tz.clone()))
                            .class(cosmic::theme::Button::Text)
                            .width(cosmic::iced::Length::Fill),
                    );
                }

                clock_box = clock_box.push(match_list);
            }

            // Per-clock format options: seconds, 24h, weekday, date.
            clock_box = clock_box
                .push(toggle_row(
                    fl!("show-seconds"),
                    clock.show_seconds,
                    move |v| Message::ToggleSeconds(index, v),
                ))
                .push(toggle_row(
                    fl!("military-time"),
                    clock.military_time,
                    move |v| Message::ToggleMilitary(index, v),
                ))
                .push(toggle_row(
                    fl!("show-weekday"),
                    clock.show_weekday,
                    move |v| Message::ToggleWeekday(index, v),
                ))
                .push(toggle_row(fl!("show-date"), clock.show_date, move |v| {
                    Message::ToggleDate(index, v)
                }));

            list = list.push(cosmic::applet::padded_control(clock_box));
        }

        let add_button = cosmic::widget::button::text(fl!("add-clock"))
            .on_press(Message::AddClock)
            .class(cosmic::theme::Button::Standard);

        list = list.push(cosmic::applet::padded_control(add_button));

        self.core
            .applet
            .popup_container(cosmic::widget::container(list))
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_time_military_and_seconds() {
        let now = Zoned::now().with_time_zone(TimeZone::UTC);
        let time = format_time(&now, "UTC", true, true, false, false);
        assert_eq!(time.len(), 8, "%H:%M:%S");
    }

    #[test]
    fn format_time_falls_back_to_utc_on_bad_timezone() {
        let now = Zoned::now().with_time_zone(TimeZone::UTC);
        // Unknown timezone falls back to UTC rather than panicking.
        let _ = format_time(&now, "Not/AZone", true, false, false, false);
    }

    #[test]
    fn format_time_with_date_matches_system_applet() {
        // Sep 27, 12:18 PM — long month abbreviation, comma, 12h clock.
        let day = jiff::civil::datetime(2026, 9, 27, 12, 18, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let time = format_time(&day, "UTC", false, false, true, false);
        assert_eq!(time, "Sep 27, 12:18 PM");
    }

    #[test]
    fn format_time_with_weekday_matches_system_applet() {
        // Sun, Sep 27, 12:18 PM — weekday only shows with the date.
        let day = jiff::civil::datetime(2026, 9, 27, 12, 18, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let time = format_time(&day, "UTC", false, false, true, true);
        assert_eq!(time, "Sun, Sep 27, 12:18 PM");
    }
}
