const std = @import("std");
const Allocator = std.mem.Allocator;
const types = @import("types.zig");
const Assertion = types.Assertion;
const AssertionResult = types.AssertionResult;
const HttpResult = types.HttpResult;

pub fn evaluateAssertions(allocator: Allocator, assertions: []const Assertion, result: *const HttpResult) !std.ArrayList(AssertionResult) {
    var assertion_results = std.ArrayList(AssertionResult).init(allocator);

    for (assertions) |assertion| {
        const assertion_result = try evaluateAssertion(allocator, assertion, result);
        try assertion_results.append(assertion_result);
    }

    return assertion_results;
}

fn evaluateAssertion(allocator: Allocator, assertion: Assertion, result: *const HttpResult) !AssertionResult {
    switch (assertion.type) {
        .response_status => {
            const expected_status = std.fmt.parseInt(u16, assertion.expected_value, 10) catch {
                return AssertionResult{
                    .assertion = assertion,
                    .passed = false,
                    .actual_value = try std.fmt.allocPrint(allocator, "{}", .{result.status_code}),
                    .error_message = try allocator.dupe(u8, "Invalid expected status code format"),
                };
            };

            const passed = result.status_code == expected_status;
            return AssertionResult{
                .assertion = assertion,
                .passed = passed,
                .actual_value = try std.fmt.allocPrint(allocator, "{}", .{result.status_code}),
                .error_message = if (!passed)
                    try std.fmt.allocPrint(allocator, "Expected status {}, got {}", .{ expected_status, result.status_code })
                else
                    null,
            };
        },

        .response_body => {
            if (result.response_body) |body| {
                const passed = std.mem.indexOf(u8, body, assertion.expected_value) != null;
                return AssertionResult{
                    .assertion = assertion,
                    .passed = passed,
                    .actual_value = try allocator.dupe(u8, body),
                    .error_message = if (!passed)
                        try std.fmt.allocPrint(allocator, "Expected body to contain '{s}'", .{assertion.expected_value})
                    else
                        null,
                };
            } else {
                return AssertionResult{
                    .assertion = assertion,
                    .passed = false,
                    .actual_value = try allocator.dupe(u8, ""),
                    .error_message = try allocator.dupe(u8, "No response body available"),
                };
            }
        },

        .response_headers => {
            if (result.response_headers) |headers| {
                const colon_pos = std.mem.indexOf(u8, assertion.expected_value, ":") orelse {
                    return AssertionResult{
                        .assertion = assertion,
                        .passed = false,
                        .actual_value = try formatHeaders(allocator, headers),
                        .error_message = try allocator.dupe(u8, "Invalid header format, expected 'Name: Value'"),
                    };
                };

                const expected_name = std.mem.trim(u8, assertion.expected_value[0..colon_pos], " \t");
                const expected_value = std.mem.trim(u8, assertion.expected_value[colon_pos + 1 ..], " \t");

                var found = false;
                for (headers) |header| {
                    if (std.mem.eql(u8, header.name, expected_name)) {
                        if (std.mem.indexOf(u8, header.value, expected_value) != null) {
                            found = true;
                            break;
                        }
                    }
                }

                return AssertionResult{
                    .assertion = assertion,
                    .passed = found,
                    .actual_value = try formatHeaders(allocator, headers),
                    .error_message = if (!found)
                        try std.fmt.allocPrint(allocator, "Expected header '{s}' with value containing '{s}'", .{ expected_name, expected_value })
                    else
                        null,
                };
            } else {
                return AssertionResult{
                    .assertion = assertion,
                    .passed = false,
                    .actual_value = try allocator.dupe(u8, ""),
                    .error_message = try allocator.dupe(u8, "No response headers available"),
                };
            }
        },
    }
}

fn formatHeaders(allocator: Allocator, headers: []const HttpResult.Header) ![]u8 {
    var formatted = std.ArrayList(u8).init(allocator);
    defer formatted.deinit();

    for (headers, 0..) |header, i| {
        if (i > 0) {
            try formatted.appendSlice(", ");
        }
        try formatted.appendSlice(header.name);
        try formatted.appendSlice(": ");
        try formatted.appendSlice(header.value);
    }

    return try allocator.dupe(u8, formatted.items);
}
