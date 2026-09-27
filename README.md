# World Clock applet for COSMIC

Shows the current time in as many timezones as you want, right in the COSMIC panel. Click the applet to add, remove, and configure clocks.

<p align="center">
    <img alt="Applet in panel" src="https://github.com/cosmic-utils/cosmic-ext-applet-world-clock/blob/main/data/applet_screenshot_1.png">
</p>

<p align="center">
    <img alt="Applet window view" src="https://github.com/cosmic-utils/cosmic-ext-applet-world-clock/blob/main/data/applet_screenshot_2.png">
</p>

<p align="center">
    <img alt="Timezone selector" src="https://github.com/cosmic-utils/cosmic-ext-applet-world-clock/blob/main/data/applet_screenshot_3.png">
</p>

## Features

- **Multiple clocks** — any number of clocks, each in its own IANA timezone
- **Searchable timezone picker** — start typing and it filters all IANA timezones
- **Custom labels** — name each clock (falls back to the timezone name when unset)
- **12/24-hour time** — switch between `3:42 PM` and `15:42`
- **Seconds** — optionally show seconds
- **Date** — optionally show today's date next to the time
- **Weekday** — optionally show the day of the week (with the date)
- **Timezone-aware** — every clock shows the correct local time for its zone, including DST transitions

## Configuration

The applet ships with a sensible default (one UTC clock) and needs no configuration. Clock set-up happens in the popup, which opens when you click the applet.

Everything is saved to the standard COSMIC config file:

```sh
~/.config/cosmic/io.github.cosmic-utils.cosmic-ext-applet-world-clock/v1/config
```

Example config:

```toml
[[clocks]]
timezone = "Europe/Warsaw"
label = "Home"

[[clocks]]
timezone = "America/New_York"

show_seconds = false
military_time = false
show_date = false
show_weekday = false
```

## Installation

### Flatpak

Depending on how you've installed COSMIC Desktop, the World Clock applet may show up in your app store by default. In COSMIC Store it should be under the "COSMIC Applets" category.

If the applet does not show up in your app store, you'll need to add `cosmic-flatpak` as a source:

```sh
flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo
```

Then proceed to your preferred app store and search for "World Clock" applet.

### Manual

The applet can be installed using the following steps:

```sh
sudo apt install libxkbcommon-dev just
git clone https://github.com/cosmic-utils/cosmic-ext-applet-world-clock.git
cd cosmic-ext-applet-world-clock
just build
sudo just install
```

`libxkbcommon-dev` is required by `smithay-client-toolkit`.

## Uninstall

To uninstall files installed by `just install`, run:

```sh
sudo just uninstall
```
