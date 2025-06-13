const std = @import("std");
const Allocator = std.mem.Allocator;

pub const HttpRequest = struct {
    method: []const u8,
    url: []const u8,
    headers: std.ArrayList(Header),
    body: ?[]const u8,

    pub const Header = struct {
        name: []const u8,
        value: []const u8,
    };

    pub fn deinit(self: *HttpRequest, allocator: Allocator) void {
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

pub const HttpResult = struct {
    status_code: u16,
    success: bool,
    error_message: ?[]const u8,
    duration_ms: u64,
};
