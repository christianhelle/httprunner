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

pub const RequestVariable = struct {
    reference: []const u8, // e.g., "login.response.body.$.token"
    request_name: []const u8,
    source: RequestVariableSource,
    target: RequestVariableTarget,
    path: []const u8, // JSONPath, XPath, header name, or "*"

    pub const RequestVariableSource = enum {
        request,
        response,
    };

    pub const RequestVariableTarget = enum {
        body,
        headers,
    };

    pub fn deinit(self: RequestVariable, allocator: Allocator) void {
        allocator.free(self.reference);
        allocator.free(self.request_name);
        allocator.free(self.path);
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
        if (self.name) |name| {
            allocator.free(name);
        }
        allocator.free(self.method);
        allocator.free(self.url);

        for (self.headers.items) |header| {
            allocator.free(header.name);
            allocator.free(header.value);
        }
        self.headers.deinit(allocator);

        if (self.body) |body| {
            allocator.free(body);
        }

        for (self.assertions.items) |assertion| {
            assertion.deinit(allocator);
        }
        self.assertions.deinit(allocator);

        for (self.variables.items) |variable| {
            variable.deinit(allocator);
        }
        self.variables.deinit(allocator);
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
        if (self.request_name) |name| {
            allocator.free(name);
        }
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
        self.assertion_results.deinit(allocator);
    }
};

pub const RequestContext = struct {
    name: []const u8,
    request: HttpRequest,
    result: ?HttpResult,

    pub fn deinit(self: *RequestContext, allocator: Allocator) void {
        allocator.free(self.name);
        self.request.deinit(allocator);
        if (self.result) |*result| {
            result.deinit(allocator);
        }
    }
};
