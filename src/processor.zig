const std = @import("std");
const print = std.debug.print;
const Allocator = std.mem.Allocator;
const time = std.time;

const colors = @import("colors.zig");
const types = @import("types.zig");
const parser = @import("parser.zig");
const runner = @import("runner.zig");
const Log = @import("log.zig").Log;

const HttpRequest = types.HttpRequest;

pub fn processHttpFiles(allocator: Allocator, files: []const []const u8, verbose: bool, log_filename: ?[]const u8, environment: ?[]const u8) !bool {
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

        const requests = parser.parseHttpFile(allocator, http_file, environment) catch |err| {
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
            requests.deinit();
        }

        if (requests.items.len == 0) {
            log.write("{s}⚠️  No HTTP requests found in file{s}\n", .{ colors.YELLOW, colors.RESET });
            continue;
        }

        log.write("Found {} HTTP request(s)\n\n", .{requests.items.len});
        files_processed += 1;

        var success_count: u32 = 0;
        var request_count: u32 = 0;
        for (requests.items) |request| {
            request_count += 1;

            if (verbose) {
                log.write("\n{s}📤 Request Details:{s}\n", .{ colors.BLUE, colors.RESET });
                log.write("Method: {s}\n", .{request.method});
                log.write("URL: {s}\n", .{request.url});

                if (request.headers.items.len > 0) {
                    log.write("Headers:\n", .{});
                    for (request.headers.items) |header| {
                        log.write("  {s}: {s}\n", .{ header.name, header.value });
                    }
                }

                if (request.body) |body| {
                    log.write("Body:\n{s}\n", .{body});
                }
                log.write("{s}\n", .{"-" ** 30});
            }

            const result = runner.executeHttpRequest(allocator, request, verbose) catch |err| {
                log.write("{s}❌ {s} {s} - Error: {}{s}\n", .{ colors.RED, request.method, request.url, err, colors.RESET });
                continue;
            };
            defer {
                var mut_result = result;
                mut_result.deinit(allocator);
            }

            if (result.success) {
                success_count += 1;
                log.write("{s}✅ {s} {s} - Status: {} - {}ms{s}\n", .{ colors.GREEN, request.method, request.url, result.status_code, result.duration_ms, colors.RESET });
            } else {
                if (result.error_message) |msg| {
                    log.write("{s}❌ {s} {s} - Status: {} - {}ms - Error: {s}{s}\n", .{ colors.RED, request.method, request.url, result.status_code, result.duration_ms, msg, colors.RESET });
                } else {
                    log.write("{s}❌ {s} {s} - Status: {} - {}ms{s}\n", .{ colors.RED, request.method, request.url, result.status_code, result.duration_ms, colors.RESET });
                }
            }

            if (request.assertions.items.len > 0) {
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
        log.write("Total requests: {s}{}{s}/{}\n", .{ if (total_success_count == total_request_count) colors.GREEN else if (total_success_count > 0) colors.YELLOW else colors.RED, total_success_count, colors.RESET, total_request_count });
    }

    return total_success_count == total_request_count;
}
