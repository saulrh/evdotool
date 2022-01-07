print(string.format("| %13s | %60s | %20s | %9s |",
                    "friendly name",
                    "name",
                    "uniq",
                    "id"))

print(string.format("| %13s | %60s | %20s | %9s |",
                    string.rep("-", 13),
                    string.rep("-", 60),
                    string.rep("-", 20),
                    string.rep("-", 9)))

for _, dev in pairs(list_all_devices()) do
   print(string.format("| %13s | %60s | %20s | %04x:%04x |",
                       dev:friendly_name(),
                       dev:name(),
                       dev:uniq(),
                       dev:vendor_id(),
                       dev:product_id()))
end
