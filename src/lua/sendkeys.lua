local inspect = require("inspect")

function sendkeys(keys, value)
   for _, key in ipairs(keys) do
      sendkey(key, value)
      sleep(0.03)
   end
end
