const std = @import("std");
const Allocator = std.mem.Allocator;
const types = @import("types.zig");
const HttpRequest = types.HttpRequest;

pub fn parseHttpFile(allocator: Allocator, file_path: []const u8) !std.ArrayList(HttpRequest) {
    const file = std.fs.cwd().openFile(file_path, .{}) catch |err| {
        return err;
    };
    defer file.close();

    const file_size = try file.getEndPos();
    const content = try allocator.alloc(u8, file_size);
    defer allocator.free(content);
    _ = try file.readAll(content);
    var requests = std.ArrayList(HttpRequest).init(allocator);
    var lines = std.mem.splitSequence(u8, content, "\n");

    var current_request: ?HttpRequest = null;
    var in_body = false;
    var body_lines = std.ArrayList([]const u8).init(allocator);
    defer body_lines.deinit();

    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t\r\n");

        // Skip empty lines and comments
        if (trimmed.len == 0 or std.mem.startsWith(u8, trimmed, "#")) {
            continue;
        }

        // Check if this is a new HTTP request line
        if (std.mem.indexOf(u8, trimmed, "HTTP/") != null or
            std.mem.startsWith(u8, trimmed, "GET ") or
            std.mem.startsWith(u8, trimmed, "POST ") or
            std.mem.startsWith(u8, trimmed, "PUT ") or
            std.mem.startsWith(u8, trimmed, "DELETE ") or
            std.mem.startsWith(u8, trimmed, "PATCH "))
        {

            // Save previous request if exists
            if (current_request) |*req| {
                if (body_lines.items.len > 0) {
                    const body = try std.mem.join(allocator, "\n", body_lines.items);
                    req.body = body;
                }
                try requests.append(req.*);
                body_lines.clearRetainingCapacity();
            }
            // Parse new request
            var parts = std.mem.splitSequence(u8, trimmed, " ");
            const method = parts.next() orelse return error.InvalidRequest;
            const url = parts.next() orelse return error.InvalidRequest;

            current_request = HttpRequest{
                .method = try allocator.dupe(u8, method),
                .url = try allocator.dupe(u8, url),
                .headers = std.ArrayList(HttpRequest.Header).init(allocator),
                .body = null,
            };
            in_body = false;
        } else if (std.mem.indexOf(u8, trimmed, ":") != null and !in_body) { // Parse header
            if (current_request) |*req| {
                var header_parts = std.mem.splitSequence(u8, trimmed, ":");
                const name = std.mem.trim(u8, header_parts.next() orelse "", " \t");
                const value = std.mem.trim(u8, header_parts.rest(), " \t");

                try req.headers.append(.{
                    .name = try allocator.dupe(u8, name),
                    .value = try allocator.dupe(u8, value),
                });
            }
        } else {
            // Body content
            in_body = true;
            try body_lines.append(try allocator.dupe(u8, trimmed));
        }
    }

    // Save last request
    if (current_request) |*req| {
        if (body_lines.items.len > 0) {
            const body = try std.mem.join(allocator, "\n", body_lines.items);
            req.body = body;
        }
        try requests.append(req.*);
    }

    return requests;
}
