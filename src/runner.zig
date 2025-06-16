const std = @import("std");
const http = std.http;
const Allocator = std.mem.Allocator;
const types = @import("types.zig");
const assertions = @import("assertions.zig");
const HttpRequest = types.HttpRequest;
const HttpResult = types.HttpResult;

pub fn executeHttpRequest(allocator: Allocator, request: HttpRequest, verbose: bool) !HttpResult {
    const start_time = std.time.nanoTimestamp();
    const has_assertions = request.assertions.items.len > 0;

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
            .assertion_results = std.ArrayList(types.AssertionResult).init(allocator),
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
            .assertion_results = std.ArrayList(types.AssertionResult).init(allocator),
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
            .assertion_results = std.ArrayList(types.AssertionResult).init(allocator),
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
    var success = status_code >= 200 and status_code < 300;

    var response_headers: ?[]types.HttpResult.Header = null;
    var response_body: ?[]const u8 = null;

    // Always capture headers and body if we have assertions or if verbose mode is enabled
    if (verbose or has_assertions) {
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

    // Evaluate assertions
    var assertion_results = std.ArrayList(types.AssertionResult).init(allocator);
    if (has_assertions) {
        const temp_result = HttpResult{
            .status_code = status_code,
            .success = success,
            .error_message = null,
            .duration_ms = duration_ms,
            .response_headers = response_headers,
            .response_body = response_body,
            .assertion_results = std.ArrayList(types.AssertionResult).init(allocator),
        };

        assertion_results = try assertions.evaluateAssertions(allocator, request.assertions.items, &temp_result);

        // Update success based on assertion results
        var all_assertions_passed = true;
        for (assertion_results.items) |result| {
            if (!result.passed) {
                all_assertions_passed = false;
                break;
            }
        }
        success = success and all_assertions_passed;
    }

    return HttpResult{
        .status_code = status_code,
        .success = success,
        .error_message = null,
        .duration_ms = duration_ms,
        .response_headers = response_headers,
        .response_body = response_body,
        .assertion_results = assertion_results,
    };
}
