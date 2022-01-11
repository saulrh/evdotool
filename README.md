# evdotool

evdotool is a tool for creating flexible virtual input devices using
the Linux evdev system. You write a lua script, bind callbacks to
device events, and dispatch events to your virtual device.

```lua
local friendly_name = args[1] or "volleyball"
local joy = find_device_by_friendly_name{friendly_name=friendly_name}

local function meta(key, value)
    sendkeys({"KEY_LEFTMETA", key}, value)
end

bind(joy, "BTN_TL", function(value) sendkey("BTN_LEFT", value) end)
bind(joy, "BTN_TL2", function(value) sendkey("BTN_RIGHT", value) end)

bind(joy, "BTN_DPAD_UP", function(value) meta("KEY_K", value) end)
bind(joy, "BTN_DPAD_DOWN", function(value) meta("KEY_J", value) end)
bind(joy, "BTN_DPAD_LEFT", function(value) meta("KEY_H", value) end)
bind(joy, "BTN_DPAD_RIGHT", function(value) meta("KEY_SEMICOLON", value) end)

local x_info = joy:axis_info("ABS_X")
local y_info = joy:axis_info("ABS_Y")

bind(joy, "ABS_X", function(value) INPUT:set_x_vel(1000 * value / x_info.maximum))
bind(joy, "ABS_Y", function(value) INPUT:set_y_vel(1000 * value / y_info.maximum))
```

See the `examples/` directory for more.

## Basic use

Clone and `cargo run path_to_script.lua -- script_args`. You can also
install with `cargo install –path .`. You will need read-write access
to evdev devices (nodes under `/dev/input`), which means either `root`
or being part of an `input` group.

## Global bindings

evdotool provides access to your lua scripts by placing a number of
objects and functions in the global namespace.

### `CODES`

A sequence of all known evdev event codes as strings. Event codes have
names like `"BTN_LEFT"` (left mouse button), `"KEY_T"` (keyboard T
key), `"ABS_X"` (joystick X value), and `"REL_X"` (mouse X movement).

### `INPUT`

A userdata handle to a synthetic input device that the script can use
to send inputs. It has the following methods:

#### `INPUT:move_x(dx)`
#### `INPUT:move_y(dy)`

Instantaneously move the mouse in the X or Y directions.

#### `INPUT:button(code, value)`

Set the given button to the given state. `code` is a string event code
identifying the button. `value` is `1` to press the button or key and
`0` to release.

#### `INPUT:set_x_vel(value)`

Set the velocity of the simulated mouse. `evdotool` runs an event loop
in the background so you don't have to handle it yourself. Dithering
*is* supported, so feel free to set fractional axis velocities!

### `DEVICES`
	
A sequence of evdev device userdata objects with the following methods:

#### `device:name()`

The name of the device (e.g. "Nintendo Switch Right Joy-Con")

#### `device:friendly_name()`

A stable identifier that you can use to identify devices for
scripts. This is a function of the device's name, product id, vendor
id, and uniq value, so it will remain the same across multiple
invocations of the tool. Friendly names are *not* guaranteed to be
unique, but there are 1200 words in the wordlist so it should be fine.

#### `device:vendor_id()`

The device's vendor id as a number.

#### `device:product_id()`

The device's product id as a number.

#### `device:axis_info(axis)`

A table of information about the given axis:

- `value`: Current axis value.
- `minimum`: Minimum axis value as reported by the device.
- `maximum`: Maximum axis value as reported by the device.
- `fuzz`: Something about touch input sensitivity, I'm not sure. It
  may be up to the application to honor it.
- `flat`: Size of the dead zone. It may be up to the application to
  honor it.
- `resolution`: Resolution of the sensor, as reported by the device.

### `bind(device, axis, callback)`

When the given device receives the specified event, the callback will
be called with a single argument that is the current value of the axis
or the current state of the button (1 pressed, 0 released).

### `sleep(seconds)`

Sleep for the given number of seconds. Accepts fractional values.

### `find_device_by_friendly_name{friendly_name=fname}`

Convenience function to select a device from the DEVICES table using
its friendly name.

### `find_device_by_ids{vendor_id=vid, product_id=pid, uniq=u}`

Convenience function to select a device from the DEVICES table using
its various IDs. `vendor_id` and `product_id` are required but `uniq`
will only be considered if non-`Nil`.

### `sendkey(code, value)`

Convenience method that's the same as calling `INPUT:button(code,
value)`.

### `sendkeys(key_sequence, value)`

Convenience method that invokes `sendkey` on each of the provided keys
with short pauses (~30 ms) between each key.

## Why? How does evdotool compare to other solutions?

There are a number of existing solutions for the problem of remapping
an input device. For example:

  - qjoypad
  - antimicro
  - joy2key
  - xboxdrv --evdev

These tools offer declarative configuration: You create a config file
that says which buttons become which other keys and which axes to use
for which mouse movements. This is simple and easy for users, to the
point that some of these tools offer configuration GUIs where you can
build macros by example.

evdotool instead offers a scripting environment in which you construct
your own mapping program. This certainly isn't as easy as playing back
example input sequences, but it is far more powerful. Specific use
cases that evdotool can handle that other systems have trouble with:

- coordinated use of multiple devices (e.g. a shift button on one
  device that modifies the axis behavior of the joystick on another
  device)
- stateful mappings (layers)
- runtime reconfiguration (using a slider to set the sensitivity of
  another axis)
