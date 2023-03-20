# Desktop Dye

DesktopDye is an open source project written in Rust that allows users to have their lights paired with Home Assistant adjust to the most dominant color on their computer screen, similar to how Philips Ambilight works with their TVs. This README file provides instructions on how to install and use DesktopDye.

## Usage

### Install DesktopDye

1. Install Rust if you haven't already. You can download it from the official Rust website: https://www.rust-lang.org/tools/install.
1. Clone the repository from GitHub using git clone https://github.com/jeroen-meijer/desktop_dye.
1. Navigate to the cli folder using `cd desktop_dye/cli`, and then install the DesktopDye CLI by running `cargo install --path .`.
1. Run the cli once using desktop_dye_cli. This will create a config file in `<USER_DIR>/.desktop_dye/config.yaml`. For Windows users, the user directory is usually `C:\Users\<USERNAME>\.desktop_dye\`. For macOS/Linux users, this will most likely be `~/.desktop_dye/`.

### Set up Home Assistant

Four things need to be done to set up Home Assistant to support DesktopDye.

#### Create an access token.

1. Go to your Home Assistant dashboard and log in.
1. Then, go to your profile, scroll down and create a new Home Assistant access token. Copy the token and save it somewhere for use later.

#### Create a light group (optional, but recommended).

This light group will contain the lights that will change based on the screen's color.

1. In your Home Assistant dashboard, go to Settings -> Devices & Services -> Helpers, and create a new helper of type Group -> Light group. Give it any name you want, then add the lights that you want to control with DesktopDye. We recommend keeping "Hide members" turned OFF.
1. Make a note of the entity ID of your newly created light group and save it somewhere for later.

#### Create a text input helper.

This will be the text field that DesktopDye will send new color values to, which an automation (made in the next step) will use to control the light group made in the previous step.

In the same Helpers menu as before, create a Helper of type Text. Give it an easily recognizable name and take note of the entity ID for later.

#### Create an automation.

This is the automation that will be triggered when the Text helper's value is changed by DesktopDye, and should change the state of the light group from before to the values in the text field.

1. In your Home Assistant dashboard, navigate to Settings -> Automations & Scenes, and create a new automation. Name it something easy to remember.
2. Click the menu in the top right, select "Edit in YAML" and paste the following template into the YAML editor. **Edit the `input_text.YOUR_TEXT_INPUT_ENTITY` and `light.YOUR_LIGHT_GROUP` parts to match the entity IDs of your text input helper and light group, respectively.**

```yaml
alias: 'Desktop Dye Client'
description: ''
trigger:
  - platform: state
    entity_id:
      - input_text.YOUR_TEXT_INPUT_ENTITY
condition: []
action:
  - service: light.turn_on
    target:
      entity_id: light.YOUR_LIGHT_GROUP
    data:
      transition: 2
      rgb_color:
        - >-
          {{ states.input_text.YOUR_TEXT_INPUT_ENTITY.state.split("
          ")[0].split(",")[0] }}
        - >-
          {{ states.input_text.YOUR_TEXT_INPUT_ENTITY.state.split("
          ")[0].split(",")[1] }}
        - >-
          {{ states.input_text.YOUR_TEXT_INPUT_ENTITY.state.split("
          ")[0].split(",")[2] }}
      brightness_pct: >-
        {{ states.input_text.YOUR_TEXT_INPUT_ENTITY.state.split("
        ")[0].split(",")[3] }}
mode: single
```

### Edit the config file

1. Open the config file located in `<USER_DIR>/.desktop_dye/config.yaml`. Again, for Windows users, this will most likely be `C:\Users\<USERNAME>\.desktop_dye\`. For macOS/Linux users, this will most likely be `~/.desktop_dye/`.
1. Update the following settings in the config file:

- `ha_target_entity_id`: Enter the entity ID of the light group helper [you created earlier](#create-a-light-group-optional-but-recommended)
- `ha_token`: Enter the Home Assistant access token [you created earlier](#create-an-access-token)
- `ha_endpoint`: Verify that this is the correct endpoint for your Home Assistant setup. By default, this should be `http://homeassistant.local:8123`.

1. Save and close the config file.

### Done! 🎉

Congratulations, you have successfully set up DesktopDye! You can now run the CLI by running `desktop_dye_cli` in your terminal. If you want to run it in the background, you can use a tool like [tmux](https://github.com/tmux/tmux).

## The Config File

The DesktopDye config file contains various settings used to control the behavior of the CLI. The config file is located in `<USER_DIR>/.desktop_dye/config.yaml`. Again, for Windows users, this will most likely be `C:\Users\<USERNAME>\.desktop_dye\`. For macOS/Linux users, this will most likely be `~/.desktop_dye/`.

The [default config file](./api/assets/default_config.yaml) contains fields for all available settings, and looks like this:

```yaml
# The ID of the screen to capture colors from.
# Is optional. If not specified, the screen primary screen is used.
# Run the application to see a list of screens and their IDs.
screen_id:

# The endpoint of the Home Assistant instance to send the colors to.
# This can usually be set to http://homeassistant.local:8123
# Is required. If not specified, the application will not start.
ha_endpoint: http://homeassistant.local:8123

# The API token to use to authenticate with Home Assistant.
# Is required. If not specified, the application will not start.
ha_token:

# The Home Assistant entity ID to send the colors to.
# This should be a text helper entity, and will usually start with 'input_text.'
# Is required. If not specified, the application will not start.
ha_target_entity_id:

# The amount of colors to sample and send to Home Assistant.
#
# Note that, depending on the algorithm used, the amount of colors sent may be less than
# this value.
#
# Is optional. If not specified, the application will sample 3 colors per capture.
# Must be between 1 and 10 (inclusive).
sample_size: 3

# The algorithm to use to select the colors to send to Home Assistant.
#
# There are two algorithms:
# - 'color_thief': A fast algorithm that selects the most dominant colors.
#   This is the default algorithm, and is relatively accurate for most use cases.
# - 'pigmnts': A slower algorithm based on the kmeans-algorithm that selects the
#    most dominant colors. Though generally slightly more accurate, it is also
#    significantly slower and not recommended for most use cases.
#
# Is optional. If not specified, the default 'color_thief' algorithm is used.
algorithm: color_thief

# The amount of seconds to wait between color captures.
#
# In other words, every this many seconds, the application will capture the colors on the
# screen, and send them to Home Assistant (if it is not already sending colors, and if the
# colors have changed).
#
# Is optional. If not specified, the application will capture colors every 3 seconds.
capture_interval: 3.0

# Selects the mode of color selection.
#
# This determines what colors are sent to Home Assistant, and in what order.
#
# There are three modes:
# - 'default': Orders the colors by occurrence, with the most dominant color first.
# - 'brightest': Orders the colors by brightness, with the brightest color first.
# - 'hue_shift': A special mode where a set of hues around the most dominant color are sent,
#    creating a gradient. In this mode, the primary color is not sent first, and may not
#    be sent at if the amount of colors in the gradient is an even number.
#
# Is optional. If not specified, the default mode is used.
mode: default

# Determines the amount of degrees of shift to use when selecting hues.
#
# This is only used when the mode is set to 'hues'.
#
# This value is used to shift the primary color to the left and right, creating a gradient.
# For example, if this is set to 45.0, the primary color will be shifted 45 degrees to the
# left and right, creating a gradient with a width of 90 degrees (with the primary color
# in the middle).
#
# Is optional. If not specified, the default value of 45.0 degrees is used.
hue_shift: 45.0

# Determines the format in which the colors are sent to Home Assistant.
#
# There are two formats:
# - 'rgb': The colors are sent as a comma-separated list of RGB values.
#   RGB values consist of three components; red, green, and blue,
#   and are integers in the range of 0-255 (inclusive).
#   Example: "255,255,0 255,0,0 0,255,0"
# - `rgbb`: The colors are sent as a comma-separated list of RGB values, with a brightness
#   component at the end. This is the default format.
#   The RGB values are integers in the range of 0-255 (inclusive), while
#   the brightness value is a float in the range of 0.0-100.0 (inclusive).
#   This is useful since Home Assistant devices generally don't support setting the brightness
#   through the RGB values, and instead require a separate brightness value.
#   Example: "255,255,0,100.0 255,0,0,100.0 0,255,0,100.0"
# - `hsb`: The colors are sent as a comma-separated list of HSB (hue, saturation, brightness)
#   values. While this format is more accurate, it is also more difficult to work with in
#   Home Assistant, and can lead to unexpected results in color accuracy. However, this is
#   the only format that supports setting a specific brightness for Home Assistant lights,
#   which the RGB format does not support.
#   HSB values are in the range of floats 0.0-360.0 for hue and floats 0.0-100.0 for the
#   saturation and brightness.
#   Example: "60.0,100.0,100.0 0.0,100.0,100.0 120.0,100.0,100.0"
#
# Is optional. If not specified, the default format (`rgbb`) is used.
color_format: rgbb

# Determines the factor by which to increase the brightness of every color.
#
# If provided, the brightness of every color is multiplied by this value. This
# is useful if the colors are too dark, or if the brightness of the colors is
# not sufficient for your lighting setup.
#
# Note that brightness values are clamped to the range of 0.0-100.0 (inclusive).
# Setting this value lower than 1.0 will decrease the brightness of the colors,
# while setting it to 0.0 will result in all colors being black.
#
# Example:
# - brightness_factor = 0.5, base brightness = 50.0, result = 25.0
# - brightness_factor = 2.0, base brightness = 50.0, result = 100.0
# - brightness_factor = 2.0, base brightness = 75.0, result = 100.0 (clamped)
#
# Is optional. If not specified, it is set to 1.0, which means no brightness
# adjustment is performed.
brightness_factor: 1.0
```

## Uninstalling

To uninstall DesktopDye, follow these steps:

1. Stop any running instances of desktop_dye_cli.
1. Run cargo uninstall desktop_dye_cli in your terminal to uninstall the CLI.
1. Delete the DesktopDye folder in the directory where you cloned the repository.
1. Delete the config file in `<USER_DIR>/.desktop_dye/config.yaml`.

## Troubleshooting

### Colors aren't changing

If the colors of the lights in your light group aren't changing, make sure that:

1. The Home Assistant automation you created is enabled and correctly set up.
1. The entity ID of your light group and your text input helper are correctly set in the config.yaml file.
1. The Home Assistant access token in the config.yaml file is correct.

### `desktop_dye_cli` won't start on macOS

If you are on macOS and desktop_dye_cli won't start, it is likely because it needs permission to capture screenshots. You can grant it permission by going to System Preferences -> Security & Privacy -> Privacy -> Screen Recording, and then adding desktop_dye_cli to the list of apps allowed to capture screenshots.

### `desktop_dye_cli` won't start on Linux

If you are on Linux and desktop_dye_cli won't start, it is likely because it needs permission to read the screen. You can grant it permission by running the following command:

```bash
$ which desktop_dye_cli
## Take note of the path to the desktop_dye_cli binary, then run the following command.

sudo setcap cap_sys_admin+ep path/to/desktop_dye_cli
## Replace path/to/desktop_dye_cli with the actual path to the binary from the first command.
```

## Contributing

If you find a bug or have a feature request, please file an issue on the [GitHub repository](https://github.com/jeroen-meijer/desktop_dye/issues).

If you want to contribute to the code, please submit a pull request. Contributions are welcome and appreciated.

## License

DesktopDye is licensed under the MIT license. See [LICENSE](./LICENSE) for more information.
