const std = @import("std");
const Allocator = std.mem.Allocator;
const types = @import("types.zig");
const HttpRequest = types.HttpRequest;
const Assertion = types.Assertion;

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
    var body_content = std.ArrayList(u8).init(allocator);
    defer body_content.deinit();

    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t\r\n");

        if (trimmed.len == 0 or std.mem.startsWith(u8, trimmed, "#")) {
            continue;
        }

        // Check for assertion directives
        if (std.mem.startsWith(u8, trimmed, "EXPECTED_RESPONSE_STATUS ")) {
            if (current_request) |*req| {
                const status_str = std.mem.trim(u8, trimmed[25..], " \t");
                try req.assertions.append(.{
                    .type = .response_status,
                    .expected_value = try allocator.dupe(u8, status_str),
                });
            }
            continue;
        } else if (std.mem.startsWith(u8, trimmed, "EXPECTED_RESPONSE_BODY ")) {
            if (current_request) |*req| {
                var body_value = std.mem.trim(u8, trimmed[23..], " \t");
                // Remove surrounding quotes if present
                if (body_value.len >= 2 and body_value[0] == '"' and body_value[body_value.len - 1] == '"') {
                    body_value = body_value[1 .. body_value.len - 1];
                }
                try req.assertions.append(.{
                    .type = .response_body,
                    .expected_value = try allocator.dupe(u8, body_value),
                });
            }
            continue;
        } else if (std.mem.startsWith(u8, trimmed, "EXPECTED_RESPONSE_HEADERS ")) {
            if (current_request) |*req| {
                var headers_value = std.mem.trim(u8, trimmed[26..], " \t");
                // Remove surrounding quotes if present
                if (headers_value.len >= 2 and headers_value[0] == '"' and headers_value[headers_value.len - 1] == '"') {
                    headers_value = headers_value[1 .. headers_value.len - 1];
                }
                try req.assertions.append(.{
                    .type = .response_headers,
                    .expected_value = try allocator.dupe(u8, headers_value),
                });
            }
            continue;
        }

        if (std.mem.indexOf(u8, trimmed, "HTTP/") != null or
            std.mem.startsWith(u8, trimmed, "GET ") or
            std.mem.startsWith(u8, trimmed, "POST ") or
            std.mem.startsWith(u8, trimmed, "PUT ") or
            std.mem.startsWith(u8, trimmed, "DELETE ") or
            std.mem.startsWith(u8, trimmed, "PATCH "))
        {
            if (current_request) |*req| {
                if (body_content.items.len > 0) {
                    req.body = try allocator.dupe(u8, body_content.items);
                }
                try requests.append(req.*);
                body_content.clearRetainingCapacity();
            }
            var parts = std.mem.splitSequence(u8, trimmed, " ");
            const method = parts.next() orelse return error.InvalidRequest;
            const url = parts.next() orelse return error.InvalidRequest;

            current_request = HttpRequest{
                .method = try allocator.dupe(u8, method),
                .url = try allocator.dupe(u8, url),
                .headers = std.ArrayList(HttpRequest.Header).init(allocator),
                .body = null,
                .assertions = std.ArrayList(Assertion).init(allocator),
            };
            in_body = false;
        } else if (std.mem.indexOf(u8, trimmed, ":") != null and !in_body) {
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
            in_body = true;
            if (body_content.items.len > 0) {
                try body_content.append('\n');
            }
            try body_content.appendSlice(trimmed);
        }
    }

    if (current_request) |*req| {
        if (body_content.items.len > 0) {
            req.body = try allocator.dupe(u8, body_content.items);
        }
        try requests.append(req.*);
    }

    return requests;
}
