function find_device_by_ids(t)
   local vendor_id = assert(t.vendor_id)
   local product_id = assert(t.product_id)
   local uniq = t.uniq

   for _, dev in pairs(DEVICES) do
      local vendor_matches = dev:vendor_id() == vendor_id
      local product_matches = dev:product_id() == product_id
      local uniq_matches = vendor:uniq() == uniq
      if vendor_matches and product_matches and (not uniq or uniq_matches) then
         return dev
      end
   end
end
