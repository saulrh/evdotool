local MAX_SPEED = 1000

local friendly_name = arg[1]
local joy = find_device_by_friendly_name{friendly_name=friendly_name}
if not joy then
   error "Did not find a joycon with that friendly name!"
end

local function with_win(key, value)
   sendkeys({"KEY_LEFTMETA", key}, value)
end

-- Left joycon triggers, top and bottom. Also right joycon SL and SR,
-- but we'll ignore those.
bind(joy, "BTN_TL", function(value) sendkey("BTN_LEFT", value) end)
bind(joy, "BTN_TL2", function(value) sendkey("BTN_RIGHT", value) end)

-- Right joycon triggers, top and bottom. Also right joycon SL and SR,
-- but we'll ignore those.
bind(joy, "BTN_TR", function(value) sendkey("BTN_LEFT", value) end)
bind(joy, "BTN_TR2", function(value) sendkey("BTN_RIGHT", value) end)

-- Left joycon face buttons
bind(joy, "BTN_DPAD_UP", function(value) with_win("KEY_K", value) end)
bind(joy, "BTN_DPAD_DOWN", function(value) with_win("KEY_J", value) end)
bind(joy, "BTN_DPAD_LEFT", function(value) with_win("KEY_H", value) end)
bind(joy, "BTN_DPAD_RIGHT", function(value) with_win("KEY_SEMICOLON", value) end)

-- Right joycon face buttons
bind(joy, "BTN_NORTH", function(value) with_win("KEY_K", value) end)
bind(joy, "BTN_SOUTH", function(value) with_win("KEY_J", value) end)
bind(joy, "BTN_WEST", function(value) with_win("KEY_H", value) end)
bind(joy, "BTN_EAST", function(value) with_win("KEY_SEMICOLON", value) end)

-- plus and minus buttons
bind(joy, "BTN_MODE", function(value) with_win("KEY_N", value) end)
bind(joy, "BTN_SELECT", function(value) with_win("KEY_N", value) end)

-- X is left joycon side to side, RX is right joycon side to side
local y_info = joy:axis_info("ABS_Y") or joy:axis_info("ABS_RY");
-- Y is left joycon up and down, RY is right joycon up and down
local x_info = joy:axis_info("ABS_X") or joy:axis_info("ABS_RX");

local function make_axis_handler(setter)
   return function (value)
      if value > 5000 then
         setter(MAX_SPEED * value / x_info.maximum)
      elseif value < -5000 then
         setter(-MAX_SPEED * value / x_info.minimum)
      else
         setter(0)
      end
   end
end

bind(joy, "ABS_X", make_axis_handler(function(s) INPUT:set_x_vel(s) end))
bind(joy, "ABS_RX", make_axis_handler(function(s) INPUT:set_x_vel(s) end))

bind(joy, "ABS_Y", make_axis_handler(function(s) INPUT:set_y_vel(s) end))
bind(joy, "ABS_RY", make_axis_handler(function(s) INPUT:set_y_vel(s) end))
