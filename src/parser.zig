const std = @import("std");
const Allocator = std.mem.Allocator;
const types = @import("types.zig");
const environment = @import("environment.zig");
const HttpRequest = types.HttpRequest;
const Assertion = types.Assertion;
const Variable = types.Variable;

pub fn parseHttpFile(allocator: Allocator, file_path: []const u8, environment_name: ?[]const u8) !std.ArrayList(HttpRequest) {
    const file = std.fs.cwd().openFile(file_path, .{}) catch |err| {
        return err;
    };
    defer file.close();

    const file_size = try file.getEndPos();
    const content = try allocator.alloc(u8, file_size);
    defer allocator.free(content);
    _ = try file.readAll(content);
    var requests = std.ArrayList(HttpRequest).init(allocator);

    var env_variables = environment.loadEnvironmentFile(allocator, file_path, environment_name) catch blk: {
        // If environment loading fails, continue without environment variables
        break :blk std.ArrayList(Variable).init(allocator);
    };
    defer {
        for (env_variables.items) |variable| {
            variable.deinit(allocator);
        }
        env_variables.deinit();
    }

    var variables = std.ArrayList(Variable).init(allocator);
    defer {
        for (variables.items) |variable| {
            variable.deinit(allocator);
        }
        variables.deinit();
    }

    for (env_variables.items) |env_var| {
        try variables.append(.{
            .name = try allocator.dupe(u8, env_var.name),
            .value = try allocator.dupe(u8, env_var.value),
        });
    }
    var lines = std.mem.splitSequence(u8, content, "\n");

    var current_request: ?HttpRequest = null;
    var in_body = false;
    var body_content = std.ArrayList(u8).init(allocator);
    defer body_content.deinit();
    var pending_request_name: ?[]const u8 = null;

    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t\r\n");
        if (trimmed.len == 0) {
            continue;
        }

        if (std.mem.startsWith(u8, trimmed, "# @name ") and trimmed.len > 8) {
            const name = std.mem.trim(u8, trimmed[8..], " \t");
            if (pending_request_name) |old_name| {
                allocator.free(old_name);
            }
            pending_request_name = try allocator.dupe(u8, name);
            continue;
        } else if (std.mem.startsWith(u8, trimmed, "// @name ") and trimmed.len > 9) {
            const name = std.mem.trim(u8, trimmed[9..], " \t");
            if (pending_request_name) |old_name| {
                allocator.free(old_name);
            }
            pending_request_name = try allocator.dupe(u8, name);
            continue;
        }

        if (std.mem.startsWith(u8, trimmed, "#") or std.mem.startsWith(u8, trimmed, "//")) {
            continue;
        }

        if (std.mem.startsWith(u8, trimmed, "@")) {
            if (std.mem.indexOf(u8, trimmed, "=")) |eq_pos| {
                const var_name = std.mem.trim(u8, trimmed[1..eq_pos], " \t");
                const var_value = std.mem.trim(u8, trimmed[eq_pos + 1 ..], " \t");

                const substituted_value = try substituteVariables(allocator, var_value, variables.items);

                var found = false;
                for (variables.items) |*variable| {
                    if (std.mem.eql(u8, variable.name, var_name)) {
                        allocator.free(variable.value);
                        variable.value = substituted_value;
                        found = true;
                        break;
                    }
                }

                if (!found) {
                    try variables.append(.{
                        .name = try allocator.dupe(u8, var_name),
                        .value = substituted_value,
                    });
                }
            }
            continue;
        }

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
                    const substituted_body = try substituteVariables(allocator, body_content.items, variables.items);
                    req.body = substituted_body;
                }
                try requests.append(req.*);
                body_content.clearRetainingCapacity();
            }
            var parts = std.mem.splitSequence(u8, trimmed, " ");
            const method = parts.next() orelse return error.InvalidRequest;
            const url = parts.next() orelse return error.InvalidRequest;

            const substituted_method = try substituteVariables(allocator, method, variables.items);
            const substituted_url = try substituteVariables(allocator, url, variables.items);

            current_request = HttpRequest{
                .name = pending_request_name,
                .method = substituted_method,
                .url = substituted_url,
                .headers = std.ArrayList(HttpRequest.Header).init(allocator),
                .body = null,
                .assertions = std.ArrayList(Assertion).init(allocator),
                .variables = std.ArrayList(Variable).init(allocator),
            };
            pending_request_name = null; // Reset after using
            in_body = false;
        } else if (std.mem.indexOf(u8, trimmed, ":") != null and !in_body) {
            if (current_request) |*req| {
                var header_parts = std.mem.splitSequence(u8, trimmed, ":");
                const name = std.mem.trim(u8, header_parts.next() orelse "", " \t");
                const value = std.mem.trim(u8, header_parts.rest(), " \t");

                const substituted_name = try substituteVariables(allocator, name, variables.items);
                const substituted_value = try substituteVariables(allocator, value, variables.items);

                try req.headers.append(.{
                    .name = substituted_name,
                    .value = substituted_value,
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
            const substituted_body = try substituteVariables(allocator, body_content.items, variables.items);
            req.body = substituted_body;
        }
        try requests.append(req.*);
    }

    // Clean up any unused pending request name
    if (pending_request_name) |name| {
        allocator.free(name);
    }

    return requests;
}

fn substituteVariables(allocator: Allocator, input: []const u8, variables: []const Variable) ![]const u8 {
    var result = std.ArrayList(u8).init(allocator);
    defer result.deinit();

    var i: usize = 0;
    while (i < input.len) {
        if (i + 1 < input.len and input[i] == '{' and input[i + 1] == '{') {
            var j = i + 2;
            while (j + 1 < input.len and !(input[j] == '}' and input[j + 1] == '}')) {
                j += 1;
            }

            if (j + 1 < input.len) {
                const var_name = input[i + 2 .. j];
                var found = false;
                for (variables) |variable| {
                    if (std.mem.eql(u8, variable.name, var_name)) {
                        try result.appendSlice(variable.value);
                        found = true;
                        break;
                    }
                }

                if (!found) {
                    try result.appendSlice(input[i .. j + 2]);
                }

                i = j + 2;
            } else {
                try result.append(input[i]);
                i += 1;
            }
        } else {
            try result.append(input[i]);
            i += 1;
        }
    }

    return try result.toOwnedSlice();
}
