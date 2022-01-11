function find_device_by_friendly_name(t)
   local friendly_name = assert(t.friendly_name)
   for _, dev in pairs(DEVICES) do
      if dev:friendly_name() == friendly_name then
         return dev
      end
   end
end
