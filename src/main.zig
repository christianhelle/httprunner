const std = @import("std");

const HttpRequest = struct {
    method: []const u8,
    url: []const u8,
    headers: std.ArrayList(Header),
    body: ?[]const u8,

    const Header = struct {
        name: []const u8,
        value: []const u8,
    };    fn deinit(self: *HttpRequest, allocator: Allocator) void {
        allocator.free(self.method);
        allocator.free(self.url);
        
        for (self.headers.items) |header| {
            allocator.free(header.name);
            allocator.free(header.value);
        }
        self.headers.deinit();
        
        if (self.body) |body| {
            allocator.free(body);
        }
    }
};

const HttpResult = struct {
    status_code: u16,
    success: bool,
    error_message: ?[]const u8,
};

pub fn main() !void {
    // Prints to stderr (it's a shortcut based on `std.io.getStdErr()`)
    std.debug.print("Hello, {s}!\n", .{"HTTP Runner"});
}

test "simple test" {
    var list = std.ArrayList(i32).init(std.testing.allocator);
    defer list.deinit(); // try commenting this out and see if zig detects the memory leak!
    try list.append(42);
    try std.testing.expectEqual(@as(i32, 42), list.pop());
}
