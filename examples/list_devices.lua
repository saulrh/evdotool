local devices = list_all_devices()

local function map(f, arr)
   local result = {}
   for k, v in pairs(arr) do
      result[k] = f(k, v)
   end
   return result
end

local function format_string_field(length, value)
   local fmt = "%" .. tostring(length) .. "s"
   return string.format(fmt, value)
end

local function format_hex_field(length, value)
   if value ~= 0 then
      return format_string_field(length, string.format("%04x", value))
   else
      return string.rep(" ", length)
   end
end

local function string_field_length(value)
   return value:len()
end

local function hex_field_length(value)
   return 4
end

local function print_columns(fields, rows)
   local lengths = {}
   for column_idx, field in pairs(fields) do
      local len = string_field_length(field.name)
      lengths[column_idx] = math.max(len, lengths[column_idx] or 0)
   end
   for _, row in pairs(rows) do
      for column_idx, value in pairs(row) do
         local len = fields[column_idx].len(value)
         lengths[column_idx] = math.max(len, lengths[column_idx] or 0)
      end
   end

   local column_names = map(function(ci, f) return format_string_field(lengths[ci], f.name) end, fields)
   local header_row = table.concat(column_names, " | ")
   print(header_row)
   print(string.rep("-", header_row:len()))
   for _, row in pairs(rows) do
      row_parts = {}
      for column_idx, value in pairs(row) do
         row_parts[#row_parts + 1] = fields[column_idx].format(lengths[column_idx], value)
      end
      print(table.concat(row_parts, " | "))
   end
end

local fields = {
   {name="friendly_name", len=string_field_length, format=format_string_field },
   {name="name", len=string_field_length, format=format_string_field},
   {name="uniq", len=string_field_length, format=format_string_field},
   {name="product_id", len=hex_field_length, format=format_hex_field},
   {name="vendor_id", len=hex_field_length, format=format_hex_field},
}

local rows = {}

for _, dev in pairs(devices) do
   local row = {}
   for _, field in pairs(fields) do
      row[#row + 1] = dev[field.name](dev) or ""
   end
   rows[#rows + 1] = row
end

print_columns(fields, rows)
