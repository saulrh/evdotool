local NINTENDO = 0x057e
local LEFT_JOYCON = 0x2006
local RIGHT_JOYCON = 0x2007

local MAX_SPEED = 1000

-- Left joycon face buttons
--   BTN_DPAD_UP
--   BTN_DPAD_DOWN
--   BTN_DPAD_LEFT
--   BTN_DPAD_RIGHT

-- Right joycon face buttons
--   BTN_NORTH
--   BTN_SOUTH
--   BTN_EAST
--   BTN_WEST

-- Left joycon screenshot
--   BTN_Z
-- Left joycon minus button
--   BTN_SELECT

-- Right joycon home button
--   BTN_MODE
-- Right joycon plus button
--   BTN_START

-- Left joycon triggers (also the right joycon S-buttons,
-- but we'll ignore those)
--   BTN_TL
--   BTN_TL2

-- Right joycon triggers (also the left joycon S-buttons,
-- but we'll ignore those)
--   BTN_TR
--   BTN_TR2

local function is_joycon(dev)
   local is_nintendo = dev:vendor_id() == NINTENDO
   local is_left_jc = dev:product_id() == LEFT_JOYCON
   local is_right_jc = dev:product_id() == RIGHT_JOYCON
   return is_nintendo and (is_left_jc or is_right_jc)
end

local friendly_name = arg[1]
local joy
for _, dev in pairs(DEVICES) do
   if is_joycon(dev) and dev:friendly_name() == friendly_name then
      joy = dev
   end
end

if not joy then
   error "Did not find a joycon with that friendly name!"
end

local function with_win(key)
   return function(value)
      INPUT:button("KEY_LEFTMETA", value)
      sleep(.03)
      INPUT:button(key, value)
   end
end

bind(joy, "BTN_TL", function(value) INPUT:button("BTN_LEFT", value) end)
bind(joy, "BTN_TL2", function(value) INPUT:button("BTN_RIGHT", value) end)

bind(joy, "BTN_TR", function(value) INPUT:button("BTN_LEFT", value) end)
bind(joy, "BTN_TR2", function(value) INPUT:button("BTN_RIGHT", value) end)

bind(joy, "BTN_DPAD_UP", with_win("KEY_K"))
bind(joy, "BTN_DPAD_DOWN", with_win("KEY_J"))
bind(joy, "BTN_DPAD_LEFT", with_win("KEY_H"))
bind(joy, "BTN_DPAD_RIGHT", with_win("KEY_SEMICOLON"))

bind(joy, "BTN_NORTH", with_win("KEY_K"))
bind(joy, "BTN_SOUTH", with_win("KEY_J"))
bind(joy, "BTN_WEST", with_win("KEY_H"))
bind(joy, "BTN_EAST", with_win("KEY_SEMICOLON"))

bind(joy, "BTN_MODE", with_win("KEY_N"))
bind(joy, "BTN_SELECT", with_win("KEY_N"))

local y_info = joy:axis_info("ABS_Y") or joy:axis_info("ABS_RY");
local x_info = joy:axis_info("ABS_X") or joy:axis_info("ABS_RX");

local function handle_x(value)
   if value > 5000 then
      INPUT:set_x_vel(MAX_SPEED * value / x_info.maximum)
   elseif value < -5000 then
      INPUT:set_x_vel(-MAX_SPEED * value / x_info.minimum)
   else
      INPUT:set_x_vel(0)
   end
end

local function handle_y(value)
   if value > 5000 then
      INPUT:set_y_vel(MAX_SPEED * value / y_info.maximum)
   elseif value < -5000 then
      INPUT:set_y_vel(-MAX_SPEED * value / y_info.minimum)
   else
      INPUT:set_y_vel(0)
   end
end

bind(joy, "ABS_X", handle_x)
bind(joy, "ABS_RX", handle_x)

bind(joy, "ABS_Y", handle_y)
bind(joy, "ABS_RY", handle_y)
