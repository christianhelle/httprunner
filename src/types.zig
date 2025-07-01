const std = @import("std");
const Allocator = std.mem.Allocator;

pub const Variable = struct {
    name: []const u8,
    value: []const u8,

    pub fn deinit(self: Variable, allocator: Allocator) void {
        allocator.free(self.name);
        allocator.free(self.value);
    }
};

pub const HttpRequest = struct {
    name: ?[]const u8,
    method: []const u8,
    url: []const u8,
    headers: std.ArrayList(Header),
    body: ?[]const u8,
    assertions: std.ArrayList(Assertion),
    variables: std.ArrayList(Variable),

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

        for (self.assertions.items) |assertion| {
            assertion.deinit(allocator);
        }
        self.assertions.deinit();

        for (self.variables.items) |variable| {
            variable.deinit(allocator);
        }
        self.variables.deinit();
    }
};

pub const Assertion = struct {
    type: AssertionType,
    expected_value: []const u8,

    pub const AssertionType = enum {
        response_status,
        response_body,
        response_headers,
    };

    pub fn deinit(self: Assertion, allocator: Allocator) void {
        allocator.free(self.expected_value);
    }
};

pub const AssertionResult = struct {
    assertion: Assertion,
    passed: bool,
    actual_value: ?[]const u8,
    error_message: ?[]const u8,

    pub fn deinit(self: *AssertionResult, allocator: Allocator) void {
        if (self.actual_value) |value| {
            allocator.free(value);
        }
        if (self.error_message) |msg| {
            allocator.free(msg);
        }
    }
};

pub const HttpResult = struct {
    status_code: u16,
    success: bool,
    error_message: ?[]const u8,
    duration_ms: u64,
    response_headers: ?[]Header,
    response_body: ?[]const u8,
    assertion_results: std.ArrayList(AssertionResult),
    request_name: ?[]const u8,

    pub const Header = struct {
        name: []const u8,
        value: []const u8,
    };

    pub fn deinit(self: *HttpResult, allocator: Allocator) void {
        if (self.response_headers) |headers| {
            for (headers) |header| {
                allocator.free(header.name);
                allocator.free(header.value);
            }
            allocator.free(headers);
        }
        if (self.response_body) |body| {
            allocator.free(body);
        }

        for (self.assertion_results.items) |*result| {
            result.deinit(allocator);
        }
        self.assertion_results.deinit();
    }
};
