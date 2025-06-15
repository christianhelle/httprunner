const std = @import("std");
const http = std.http;
const Allocator = std.mem.Allocator;
const types = @import("types.zig");
const HttpRequest = types.HttpRequest;
const HttpResult = types.HttpResult;

pub fn executeHttpRequest(allocator: Allocator, request: HttpRequest, verbose: bool) !HttpResult {
    const start_time = std.time.nanoTimestamp();

    const uri = std.Uri.parse(request.url) catch {
        const end_time = std.time.nanoTimestamp();
        const duration_ms = @as(u64, @intCast(@divTrunc((end_time - start_time), 1_000_000)));
        return HttpResult{
            .status_code = 0,
            .success = false,
            .error_message = "Invalid URL",
            .duration_ms = duration_ms,
            .response_headers = null,
            .response_body = null,
        };
    };

    var client = http.Client{ .allocator = allocator };
    defer client.deinit();

    const method = std.meta.stringToEnum(http.Method, request.method) orelse {
        const end_time = std.time.nanoTimestamp();
        const duration_ms = @as(u64, @intCast(@divTrunc((end_time - start_time), 1_000_000)));
        return HttpResult{
            .status_code = 0,
            .success = false,
            .error_message = "Invalid HTTP method",
            .duration_ms = duration_ms,
            .response_headers = null,
            .response_body = null,
        };
    };
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
            .response_headers = null,
            .response_body = null,
        };
    };
    defer req.deinit();

    try req.send();
    if (request.body) |body| {
        req.transfer_encoding = .chunked;
        try req.writeAll(body);
    }
    try req.finish();
    try req.wait();

    const end_time = std.time.nanoTimestamp();
    const duration_ms = @as(u64, @intCast(@divTrunc((end_time - start_time), 1_000_000)));

    const status_code = @intFromEnum(req.response.status);
    const success = status_code >= 200 and status_code < 300;

    var response_headers: ?[]types.HttpResult.Header = null;
    var response_body: ?[]const u8 = null;
    if (verbose) {
        var headers_list = std.ArrayList(types.HttpResult.Header).init(allocator);
        var header_iter = req.response.iterateHeaders();
        while (header_iter.next()) |header| {
            const name_copy = try allocator.dupe(u8, header.name);
            const value_copy = try allocator.dupe(u8, header.value);
            try headers_list.append(.{ .name = name_copy, .value = value_copy });
        }
        response_headers = try headers_list.toOwnedSlice();

        const body = try req.reader().readAllAlloc(allocator, 1024 * 1024);
        response_body = body;
    }

    return HttpResult{
        .status_code = status_code,
        .success = success,
        .error_message = null,
        .duration_ms = duration_ms,
        .response_headers = response_headers,
        .response_body = response_body,
    };
}
