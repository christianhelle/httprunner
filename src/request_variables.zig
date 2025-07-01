const std = @import("std");
const Allocator = std.mem.Allocator;
const types = @import("types.zig");
const RequestVariable = types.RequestVariable;
const RequestContext = types.RequestContext;
const HttpResult = types.HttpResult;

pub fn parseRequestVariable(allocator: Allocator, reference: []const u8) !RequestVariable {
    // Parse syntax: {{<request_name>.(request|response).(body|headers).(*|JSONPath|XPath|<header_name>)}}

    // Remove surrounding {{ and }}
    var cleaned = reference;
    if (std.mem.startsWith(u8, reference, "{{") and std.mem.endsWith(u8, reference, "}}")) {
        cleaned = reference[2 .. reference.len - 2];
    }

    var parts = std.mem.splitSequence(u8, cleaned, ".");

    const request_name = parts.next() orelse return error.InvalidRequestVariable;
    const source_str = parts.next() orelse return error.InvalidRequestVariable;
    const target_str = parts.next() orelse return error.InvalidRequestVariable;
    const path = parts.rest();

    const source = if (std.mem.eql(u8, source_str, "request"))
        RequestVariable.RequestVariableSource.request
    else if (std.mem.eql(u8, source_str, "response"))
        RequestVariable.RequestVariableSource.response
    else
        return error.InvalidRequestVariable;

    const target = if (std.mem.eql(u8, target_str, "body"))
        RequestVariable.RequestVariableTarget.body
    else if (std.mem.eql(u8, target_str, "headers"))
        RequestVariable.RequestVariableTarget.headers
    else
        return error.InvalidRequestVariable;

    return RequestVariable{
        .reference = try allocator.dupe(u8, reference),
        .request_name = try allocator.dupe(u8, request_name),
        .source = source,
        .target = target,
        .path = try allocator.dupe(u8, path),
    };
}

pub fn extractRequestVariableValue(allocator: Allocator, request_var: RequestVariable, context: []const RequestContext) !?[]const u8 {
    // Find the request context by name
    var target_context: ?*const RequestContext = null;
    for (context) |*ctx| {
        if (std.mem.eql(u8, ctx.name, request_var.request_name)) {
            target_context = ctx;
            break;
        }
    }

    if (target_context == null) {
        return null; // Request not found or not executed yet
    }

    const ctx = target_context.?;

    switch (request_var.source) {
        .request => return extractFromRequest(allocator, request_var, &ctx.request),
        .response => {
            if (ctx.result) |*result| {
                return extractFromResponse(allocator, request_var, result);
            } else {
                return null; // Request not executed yet
            }
        },
    }
}

fn extractFromRequest(allocator: Allocator, request_var: RequestVariable, request: *const types.HttpRequest) !?[]const u8 {
    switch (request_var.target) {
        .body => {
            if (request.body) |body| {
                if (std.mem.eql(u8, request_var.path, "*")) {
                    return try allocator.dupe(u8, body);
                }
                // For request body, we don't support JSONPath/XPath parsing yet
                // Return the whole body for any path (including non-wildcard)
                return try allocator.dupe(u8, body);
            }
            return null;
        },
        .headers => {
            for (request.headers.items) |header| {
                if (std.ascii.eqlIgnoreCase(header.name, request_var.path)) {
                    return try allocator.dupe(u8, header.value);
                }
            }
            return null;
        },
    }
}

fn extractFromResponse(allocator: Allocator, request_var: RequestVariable, result: *const HttpResult) !?[]const u8 {
    switch (request_var.target) {
        .body => {
            if (result.response_body) |body| {
                if (std.mem.eql(u8, request_var.path, "*")) {
                    return try allocator.dupe(u8, body);
                }

                // Basic JSONPath support for $.property
                if (std.mem.startsWith(u8, request_var.path, "$.")) {
                    const property = request_var.path[2..];
                    return extractJsonProperty(allocator, body, property);
                }

                // If no special path, return whole body
                return try allocator.dupe(u8, body);
            }
            return null;
        },
        .headers => {
            if (result.response_headers) |headers| {
                for (headers) |header| {
                    if (std.ascii.eqlIgnoreCase(header.name, request_var.path)) {
                        return try allocator.dupe(u8, header.value);
                    }
                }
            }
            return null;
        },
    }
}

fn extractJsonProperty(allocator: Allocator, json_body: []const u8, property: []const u8) !?[]const u8 {
    // Handle nested properties like "json.token"
    var parts = std.mem.splitSequence(u8, property, ".");
    var current_json = json_body;
    var temp_json: ?[]const u8 = null;
    defer if (temp_json) |t| allocator.free(t);

    while (parts.next()) |part| {
        const result = extractSimpleJsonProperty(allocator, current_json, part) catch |err| {
            return err;
        };
        if (result) |value| {
            if (temp_json) |t| allocator.free(t);
            temp_json = value;
            current_json = value;
        } else {
            return null;
        }
    }

    if (temp_json) |value| {
        return try allocator.dupe(u8, value);
    }

    return null;
}

fn extractSimpleJsonProperty(allocator: Allocator, json_body: []const u8, property: []const u8) !?[]const u8 {
    // Simple JSON property extraction
    // Look for "property": "value" or "property": value

    var search_pattern = std.ArrayList(u8).init(allocator);
    defer search_pattern.deinit();

    try search_pattern.appendSlice("\"");
    try search_pattern.appendSlice(property);
    try search_pattern.appendSlice("\":");

    if (std.mem.indexOf(u8, json_body, search_pattern.items)) |start_pos| {
        var pos = start_pos + search_pattern.items.len;

        // Skip whitespace
        while (pos < json_body.len and (json_body[pos] == ' ' or json_body[pos] == '\t' or json_body[pos] == '\n' or json_body[pos] == '\r')) {
            pos += 1;
        }

        if (pos >= json_body.len) return null;

        // Check if value is a string (starts with ")
        if (json_body[pos] == '"') {
            pos += 1; // Skip opening quote
            const value_start = pos;

            // Find closing quote (basic implementation, doesn't handle escaped quotes)
            while (pos < json_body.len and json_body[pos] != '"') {
                pos += 1;
            }

            if (pos < json_body.len) {
                return try allocator.dupe(u8, json_body[value_start..pos]);
            }
        } else if (json_body[pos] == '{') {
            // Handle object value
            var brace_count: i32 = 1;
            pos += 1;
            const value_start = pos - 1;

            while (pos < json_body.len and brace_count > 0) {
                if (json_body[pos] == '{') {
                    brace_count += 1;
                } else if (json_body[pos] == '}') {
                    brace_count -= 1;
                }
                pos += 1;
            }

            if (brace_count == 0) {
                return try allocator.dupe(u8, json_body[value_start..pos]);
            }
        } else {
            // Non-string value (number, boolean, null)
            const value_start = pos;

            // Find end of value (comma, }, ], or whitespace)
            while (pos < json_body.len and
                json_body[pos] != ',' and
                json_body[pos] != '}' and
                json_body[pos] != ']' and
                json_body[pos] != ' ' and
                json_body[pos] != '\t' and
                json_body[pos] != '\n' and
                json_body[pos] != '\r')
            {
                pos += 1;
            }

            return try allocator.dupe(u8, json_body[value_start..pos]);
        }
    }

    return null;
}

pub fn findRequestVariables(allocator: Allocator, input: []const u8) !std.ArrayList(RequestVariable) {
    var variables = std.ArrayList(RequestVariable).init(allocator);

    var i: usize = 0;
    while (i < input.len) {
        if (i + 1 < input.len and input[i] == '{' and input[i + 1] == '{') {
            var j = i + 2;
            while (j + 1 < input.len and !(input[j] == '}' and input[j + 1] == '}')) {
                j += 1;
            }

            if (j + 1 < input.len) {
                const var_ref = input[i .. j + 2];

                // Check if this looks like a request variable (contains dots)
                if (std.mem.count(u8, var_ref, ".") >= 3) {
                    const request_var = parseRequestVariable(allocator, var_ref) catch {
                        i = j + 2;
                        continue;
                    };
                    try variables.append(request_var);
                }

                i = j + 2;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    return variables;
}

pub fn substituteRequestVariables(allocator: Allocator, input: []const u8, context: []const RequestContext) ![]const u8 {
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
                const var_ref = input[i .. j + 2];

                // Check if this looks like a request variable (contains dots)
                if (std.mem.count(u8, var_ref, ".") >= 3) {
                    const request_var = parseRequestVariable(allocator, var_ref) catch {
                        // If parsing fails, keep original text
                        try result.appendSlice(var_ref);
                        i = j + 2;
                        continue;
                    };
                    defer request_var.deinit(allocator);

                    const value = extractRequestVariableValue(allocator, request_var, context) catch {
                        // If extraction fails, keep original text
                        try result.appendSlice(var_ref);
                        i = j + 2;
                        continue;
                    };

                    if (value) |val| {
                        try result.appendSlice(val);
                        allocator.free(val);
                    } else {
                        // If value not found, keep original text
                        try result.appendSlice(var_ref);
                    }
                } else {
                    // Not a request variable, keep original text
                    try result.appendSlice(var_ref);
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
