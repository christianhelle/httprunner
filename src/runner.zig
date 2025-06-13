const std = @import("std");
const http = std.http;
const Allocator = std.mem.Allocator;
const types = @import("types.zig");
const HttpRequest = types.HttpRequest;
const HttpResult = types.HttpResult;

pub fn executeHttpRequest(allocator: Allocator, request: HttpRequest) !HttpResult {
    const start_time = std.time.nanoTimestamp();

    // Parse URL to extract scheme, host, port, and path
    const uri = std.Uri.parse(request.url) catch {
        const end_time = std.time.nanoTimestamp();
        const duration_ms = @as(u64, @intCast(@divTrunc((end_time - start_time), 1_000_000)));
        return HttpResult{
            .status_code = 0,
            .success = false,
            .error_message = "Invalid URL",
            .duration_ms = duration_ms,
        };
    };

    var client = http.Client{ .allocator = allocator };
    defer client.deinit();

    // Convert method string to enum
    const method = std.meta.stringToEnum(http.Method, request.method) orelse {
        const end_time = std.time.nanoTimestamp();
        const duration_ms = @as(u64, @intCast(@divTrunc((end_time - start_time), 1_000_000)));
        return HttpResult{
            .status_code = 0,
            .success = false,
            .error_message = "Invalid HTTP method",
            .duration_ms = duration_ms,
        };
    };
    // Prepare headers
    var header_buffer: [8192]u8 = undefined;
    var req = client.open(method, uri, .{
        .server_header_buffer = &header_buffer,
    }) catch |err| {
        const end_time = std.time.nanoTimestamp();
        const duration_ms = @as(u64, @intCast(@divTrunc((end_time - start_time), 1_000_000)));
        return HttpResult{
            .status_code = 0,
            .success = false,
            .error_message = switch (err) {
                error.UnknownHostName => "Unknown host",
                error.ConnectionRefused => "Connection refused",
                error.NetworkUnreachable => "Network unreachable",
                else => "Connection error",
            },
            .duration_ms = duration_ms,
        };
    };
    defer req.deinit();

    // TODO: Add custom headers support for Zig 0.14
    // For now, we'll skip custom headers to get the basic functionality working

    try req.send();

    // Send body if present
    if (request.body) |body| {
        try req.writeAll(body);
    }
    try req.finish();
    try req.wait();

    const end_time = std.time.nanoTimestamp();
    const duration_ms = @as(u64, @intCast(@divTrunc((end_time - start_time), 1_000_000)));

    const status_code = @intFromEnum(req.response.status);
    const success = status_code >= 200 and status_code < 300;

    return HttpResult{
        .status_code = status_code,
        .success = success,
        .error_message = null,
        .duration_ms = duration_ms,
    };
}
