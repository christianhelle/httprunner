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

    var client = http.Client{ .allocator = allocator };
    defer client.deinit();

    const method = std.meta.stringToEnum(http.Method, request.method) orelse {
        const end_time = std.time.nanoTimestamp();
        const duration_ms = @as(u64, @intCast(@divTrunc((end_time - start_time), 1_000_000)));
        return HttpResult{
            .request_name = if (request.name) |name| try allocator.dupe(u8, name) else null,
            .status_code = 0,
            .success = false,
            .error_message = "Invalid HTTP method",
            .duration_ms = duration_ms,
            .response_headers = null,
            .response_body = null,
            .assertion_results = std.ArrayList(types.AssertionResult).initCapacity(allocator, 0) catch @panic("OOM"),
        };
    };

    // Simple HTTP request using fetch
    var body_buffer = std.ArrayList(u8).initCapacity(allocator, 0) catch @panic("OOM");
    defer body_buffer.deinit(allocator);

    const result = client.fetch(.{
        .method = method,
        .location = .{ .url = request.url },
    }) catch |err| {
        const end_time = std.time.nanoTimestamp();
        const duration_ms = @as(u64, @intCast(@divTrunc((end_time - start_time), 1_000_000)));
        return HttpResult{
            .request_name = if (request.name) |name| try allocator.dupe(u8, name) else null,
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
            .assertion_results = std.ArrayList(types.AssertionResult).initCapacity(allocator, 0) catch @panic("OOM"),
        };
    };

    const end_time = std.time.nanoTimestamp();
    const duration_ms = @as(u64, @intCast(@divTrunc((end_time - start_time), 1_000_000)));

    const status_code = @intFromEnum(result.status);
    var success = status_code >= 200 and status_code < 300;

    var response_headers: ?[]types.HttpResult.Header = null;
    var response_body: ?[]const u8 = null;

    if (verbose or has_assertions) {
        // Process response headers - simplified for now
        response_headers = null; // TODO: implement header processing if needed

        // Response body is in body_buffer ArrayList
        if (body_buffer.items.len > 0) {
            response_body = try allocator.dupe(u8, body_buffer.items);
        }
    }

    var assertion_results = std.ArrayList(types.AssertionResult).initCapacity(allocator, 0) catch @panic("OOM");
    if (has_assertions) {
        var temp_result = HttpResult{
            .request_name = if (request.name) |name| try allocator.dupe(u8, name) else null,
            .status_code = status_code,
            .success = success,
            .error_message = null,
            .duration_ms = duration_ms,
            .response_headers = response_headers,
            .response_body = response_body,
            .assertion_results = std.ArrayList(types.AssertionResult).initCapacity(allocator, 0) catch @panic("OOM"),
        };

        assertion_results = try assertions.evaluateAssertions(allocator, request.assertions.items, &temp_result);

        var all_assertions_passed = true;
        for (assertion_results.items) |assertion_result| {
            if (!assertion_result.passed) {
                all_assertions_passed = false;
                break;
            }
        }
        success = success and all_assertions_passed;
    }

    return HttpResult{
        .request_name = if (request.name) |name| try allocator.dupe(u8, name) else null,
        .status_code = status_code,
        .success = success,
        .error_message = null,
        .duration_ms = duration_ms,
        .response_headers = response_headers,
        .response_body = response_body,
        .assertion_results = assertion_results,
    };
}
