const std = @import("std");
const print = std.debug.print;
const Allocator = std.mem.Allocator;
const time = std.time;

const colors = @import("colors.zig");
const types = @import("types.zig");
const parser = @import("parser.zig");
const runner = @import("runner.zig");
const request_variables = @import("request_variables.zig");
const Log = @import("log.zig").Log;

const HttpRequest = types.HttpRequest;
const RequestContext = types.RequestContext;

pub fn processHttpFiles(allocator: Allocator, files: []const []const u8, verbose: bool, log_filename: ?[]const u8, environment: ?[]const u8, insecure: bool) !bool {
    var log = Log.init(allocator, log_filename) catch |err| {
        print("{s}❌ Error initializing log: {}{s}\n", .{ colors.RED, err, colors.RESET });
        return err;
    };
    defer log.deinit();

    var total_success_count: u32 = 0;
    var total_request_count: u32 = 0;
    var files_processed: u32 = 0;

    for (files) |http_file| {
        log.write("{s}🚀 HTTP File Runner - Processing file: {s}{s}\n", .{ colors.BLUE, http_file, colors.RESET });
        log.write("{s}\n", .{"=" ** 50});

        var requests = parser.parseHttpFile(allocator, http_file, environment) catch |err| {
            switch (err) {
                error.FileNotFound => {
                    log.write("{s}❌ Error: File '{s}' not found{s}\n", .{ colors.RED, http_file, colors.RESET });
                    continue;
                },
                else => {
                    log.write("{s}❌ Error parsing file: {}{s}\n", .{ colors.RED, err, colors.RESET });
                    continue;
                },
            }
        };
        defer {
            for (requests.items) |*req| {
                req.deinit(allocator);
            }
            requests.deinit(allocator);
        }

        if (requests.items.len == 0) {
            log.write("{s}⚠️  No HTTP requests found in file{s}\n", .{ colors.YELLOW, colors.RESET });
            continue;
        }

        log.write("Found {} HTTP request(s)\n\n", .{requests.items.len});
        files_processed += 1;

        var success_count: u32 = 0;
        var request_count: u32 = 0;

        // Build request contexts for sequential execution
        var request_contexts = std.ArrayList(RequestContext).initCapacity(allocator, 0) catch @panic("OOM");
        defer {
            for (request_contexts.items) |*ctx| {
                ctx.deinit(allocator);
            }
            request_contexts.deinit(allocator);
        }

        for (requests.items) |request| {
            request_count += 1;

            // Create a copy of the request for processing
            var processed_request = try cloneHttpRequest(allocator, request);
            
            // Apply global insecure flag if set
            if (insecure) {
                processed_request.insecure = true;
            }

            // Substitute request variables in the processed request
            try substituteRequestVariablesInRequest(allocator, &processed_request, request_contexts.items);

            if (verbose) {
                log.write("\n{s}📤 Request Details:{s}\n", .{ colors.BLUE, colors.RESET });
                if (processed_request.name) |name| {
                    log.write("Name: {s}\n", .{name});
                }
                log.write("Method: {s}\n", .{processed_request.method});
                log.write("URL: {s}\n", .{processed_request.url});

                if (processed_request.headers.items.len > 0) {
                    log.write("Headers:\n", .{});
                    for (processed_request.headers.items) |header| {
                        log.write("  {s}: {s}\n", .{ header.name, header.value });
                    }
                }

                if (processed_request.body) |body| {
                    log.write("Body:\n{s}\n", .{body});
                }
                log.write("{s}\n", .{"-" ** 30});
            }

            const result = runner.executeHttpRequest(allocator, processed_request, verbose) catch |err| {
                log.write("{s}❌ {s} {s} - Error: {}{s}\n", .{ colors.RED, processed_request.method, processed_request.url, err, colors.RESET });

                // Add to context even if failed
                const context_name = if (processed_request.name) |name|
                    try allocator.dupe(u8, name)
                else
                    try std.fmt.allocPrint(allocator, "request_{}", .{request_count});

                try request_contexts.append(allocator, .{
                    .name = context_name,
                    .request = processed_request,
                    .result = null,
                });
                continue;
            };

            if (result.success) {
                success_count += 1;
                const name_prefix = if (result.request_name) |name|
                    std.fmt.allocPrint(allocator, "{s}: ", .{name}) catch ""
                else
                    "";
                defer if (result.request_name != null) allocator.free(name_prefix);

                log.write("{s}✅ {s}{s} {s} - Status: {} - {}ms{s}\n", .{ colors.GREEN, name_prefix, processed_request.method, processed_request.url, result.status_code, result.duration_ms, colors.RESET });
            } else {
                const name_prefix = if (result.request_name) |name|
                    std.fmt.allocPrint(allocator, "{s}: ", .{name}) catch ""
                else
                    "";
                defer if (result.request_name != null) allocator.free(name_prefix);

                if (result.error_message) |msg| {
                    log.write("{s}❌ {s}{s} {s} - Status: {} - {}ms - Error: {s}{s}\n", .{ colors.RED, name_prefix, processed_request.method, processed_request.url, result.status_code, result.duration_ms, msg, colors.RESET });
                } else {
                    log.write("{s}❌ {s}{s} {s} - Status: {} - {}ms{s}\n", .{ colors.RED, name_prefix, processed_request.method, processed_request.url, result.status_code, result.duration_ms, colors.RESET });
                }
            }

            // Add to request context for subsequent requests
            const context_name = if (processed_request.name) |name|
                try allocator.dupe(u8, name)
            else
                try std.fmt.allocPrint(allocator, "request_{}", .{request_count});

            try request_contexts.append(allocator, .{
                .name = context_name,
                .request = processed_request,
                .result = result,
            });

            if (verbose) {
                log.write("\n{s}📥 Response Details:{s}\n", .{ colors.BLUE, colors.RESET });
                log.write("Status: {}\n", .{result.status_code});
                log.write("Duration: {}ms\n", .{result.duration_ms});

                if (result.response_headers) |headers| {
                    log.write("Headers:\n", .{});
                    for (headers) |header| {
                        log.write("  {s}: {s}\n", .{ header.name, header.value });
                    }
                }

                if (result.response_body) |body| {
                    log.write("Body:\n{s}\n", .{body});
                }
                log.write("{s}\n", .{"-" ** 30});
            }

            if (processed_request.assertions.items.len > 0) {
                log.write("\n{s}🔍 Assertion Results:{s}\n", .{ colors.BLUE, colors.RESET });
                for (result.assertion_results.items) |assertion_result| {
                    const assertion_type_str = switch (assertion_result.assertion.type) {
                        .response_status => "Status Code",
                        .response_body => "Response Body",
                        .response_headers => "Response Headers",
                    };

                    if (assertion_result.passed) {
                        log.write("{s}  ✅ {s}: Expected '{s}'{s}\n", .{ colors.GREEN, assertion_type_str, assertion_result.assertion.expected_value, colors.RESET });
                    } else {
                        log.write("{s}  ❌ {s}: {s}{s}\n", .{ colors.RED, assertion_type_str, assertion_result.error_message orelse "Failed", colors.RESET });
                        if (assertion_result.actual_value) |actual| {
                            log.write("{s}     Expected: '{s}'{s}\n", .{ colors.YELLOW, assertion_result.assertion.expected_value, colors.RESET });
                            log.write("{s}     Actual: '{s}'{s}\n", .{ colors.YELLOW, actual, colors.RESET });
                        }
                    }
                }
                log.write("{s}\n", .{"-" ** 30});
            }
        }

        log.write("\n{s}\n", .{"=" ** 50});
        log.write("File Summary: {s}{}{s}/{} requests succeeded\n\n", .{ if (success_count == request_count) colors.GREEN else if (success_count > 0) colors.YELLOW else colors.RED, success_count, colors.RESET, request_count });

        total_success_count += success_count;
        total_request_count += request_count;
    }

    if (files_processed > 1) {
        log.write("{s}🎯 Overall Summary:{s}\n", .{ colors.BLUE, colors.RESET });
        log.write("Files processed: {}\n", .{files_processed});
        log.write("Total requests: {s}{}{s}/{}\n\n", .{ if (total_success_count == total_request_count) colors.GREEN else if (total_success_count > 0) colors.YELLOW else colors.RED, total_success_count, colors.RESET, total_request_count });
    }

    return total_success_count == total_request_count;
}

fn cloneHttpRequest(allocator: Allocator, original: HttpRequest) !HttpRequest {
    var cloned = HttpRequest{
        .name = if (original.name) |name| try allocator.dupe(u8, name) else null,
        .method = try allocator.dupe(u8, original.method),
        .url = try allocator.dupe(u8, original.url),
        .headers = std.ArrayList(HttpRequest.Header).initCapacity(allocator, 0) catch @panic("OOM"),
        .body = if (original.body) |body| try allocator.dupe(u8, body) else null,
        .assertions = std.ArrayList(types.Assertion).initCapacity(allocator, 0) catch @panic("OOM"),
        .variables = std.ArrayList(types.Variable).initCapacity(allocator, 0) catch @panic("OOM"),
        .insecure = original.insecure,
    };

    for (original.headers.items) |header| {
        try cloned.headers.append(allocator, .{
            .name = try allocator.dupe(u8, header.name),
            .value = try allocator.dupe(u8, header.value),
        });
    }

    for (original.assertions.items) |assertion| {
        try cloned.assertions.append(allocator, .{
            .type = assertion.type,
            .expected_value = try allocator.dupe(u8, assertion.expected_value),
        });
    }

    for (original.variables.items) |variable| {
        try cloned.variables.append(allocator, .{
            .name = try allocator.dupe(u8, variable.name),
            .value = try allocator.dupe(u8, variable.value),
        });
    }

    return cloned;
}

fn substituteRequestVariablesInRequest(allocator: Allocator, request: *HttpRequest, context: []const RequestContext) !void {
    // Substitute in URL
    const new_url = try request_variables.substituteRequestVariables(allocator, request.url, context);
    allocator.free(request.url);
    request.url = new_url;

    // Substitute in headers
    for (request.headers.items) |*header| {
        const new_name = try request_variables.substituteRequestVariables(allocator, header.name, context);
        const new_value = try request_variables.substituteRequestVariables(allocator, header.value, context);
        allocator.free(header.name);
        allocator.free(header.value);
        header.name = new_name;
        header.value = new_value;
    }

    // Substitute in body
    if (request.body) |body| {
        const new_body = try request_variables.substituteRequestVariables(allocator, body, context);
        allocator.free(body);
        request.body = new_body;
    }

    // Substitute in assertion expected values
    for (request.assertions.items) |*assertion| {
        const new_expected = try request_variables.substituteRequestVariables(allocator, assertion.expected_value, context);
        allocator.free(assertion.expected_value);
        assertion.expected_value = new_expected;
    }
}
