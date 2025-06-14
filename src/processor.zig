const std = @import("std");
const print = std.debug.print;
const Allocator = std.mem.Allocator;
const time = std.time;

const colors = @import("colors.zig");
const types = @import("types.zig");
const parser = @import("parser.zig");
const runner = @import("runner.zig");

const HttpRequest = types.HttpRequest;

/// Custom print function that outputs to both console and log file if provided
fn customPrint(log_file: ?std.fs.File, comptime fmt: []const u8, args: anytype) void {
    print(fmt, args);
    if (log_file) |file| {
        file.writer().print(fmt, args) catch |err| {
            print("Error writing to log file: {}\n", .{err});
        };
    }
}

/// Function to create a log file with timestamp
fn createLogFile(allocator: Allocator, base_filename: []const u8) !?std.fs.File {
    if (base_filename.len == 0) return null;

    // Create a simple timestamp using seconds since epoch
    const timestamp = @as(u64, @intCast(std.time.timestamp()));

    // Format filename with timestamp
    const log_filename = try std.fmt.allocPrint(allocator, "{s}_{d}.log", .{ base_filename, timestamp });
    defer allocator.free(log_filename);

    // Create and return the file
    return try std.fs.cwd().createFile(log_filename, .{});
}

pub fn processHttpFiles(allocator: Allocator, files: []const []const u8, verbose: bool, log_filename: ?[]const u8) !void {
    // Create log file with timestamp if log_filename is provided
    var maybe_log_file: ?std.fs.File = null;
    defer if (maybe_log_file) |*file| file.close();

    if (log_filename != null) {
        maybe_log_file = try createLogFile(allocator, log_filename.?);
    }

    var total_success_count: u32 = 0;
    var total_request_count: u32 = 0;
    var files_processed: u32 = 0;

    for (files) |http_file| {
        customPrint(maybe_log_file, "{s}🚀 HTTP File Runner - Processing file: {s}{s}\n", .{ colors.BLUE, http_file, colors.RESET });
        customPrint(maybe_log_file, "{s}\n", .{"=" ** 50});

        const requests = parser.parseHttpFile(allocator, http_file) catch |err| {
            switch (err) {
                error.FileNotFound => {
                    customPrint(maybe_log_file, "{s}❌ Error: File '{s}' not found{s}\n", .{ colors.RED, http_file, colors.RESET });
                    continue;
                },
                else => {
                    customPrint(maybe_log_file, "{s}❌ Error parsing file: {}{s}\n", .{ colors.RED, err, colors.RESET });
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
            customPrint(maybe_log_file, "{s}⚠️  No HTTP requests found in file{s}\n", .{ colors.YELLOW, colors.RESET });
            continue;
        }

        customPrint(maybe_log_file, "Found {} HTTP request(s)\n\n", .{requests.items.len});
        files_processed += 1;

        var success_count: u32 = 0;
        var request_count: u32 = 0;
        for (requests.items) |request| {
            request_count += 1;

            if (verbose) {
                customPrint(maybe_log_file, "\n{s}📤 Request Details:{s}\n", .{ colors.BLUE, colors.RESET });
                customPrint(maybe_log_file, "Method: {s}\n", .{request.method});
                customPrint(maybe_log_file, "URL: {s}\n", .{request.url});

                if (request.headers.items.len > 0) {
                    customPrint(maybe_log_file, "Headers:\n", .{});
                    for (request.headers.items) |header| {
                        customPrint(maybe_log_file, "  {s}: {s}\n", .{ header.name, header.value });
                    }
                }

                if (request.body) |body| {
                    customPrint(maybe_log_file, "Body:\n{s}\n", .{body});
                }
                customPrint(maybe_log_file, "{s}\n", .{"-" ** 30});
            }

            const result = runner.executeHttpRequest(allocator, request, verbose) catch |err| {
                customPrint(maybe_log_file, "{s}❌ {s} {s} - Error: {}{s}\n", .{ colors.RED, request.method, request.url, err, colors.RESET });
                continue;
            };
            defer {
                var mut_result = result;
                mut_result.deinit(allocator);
            }

            if (result.success) {
                success_count += 1;
                customPrint(maybe_log_file, "{s}✅ {s} {s} - Status: {} - {}ms{s}\n", .{ colors.GREEN, request.method, request.url, result.status_code, result.duration_ms, colors.RESET });
            } else {
                if (result.error_message) |msg| {
                    customPrint(maybe_log_file, "{s}❌ {s} {s} - Status: {} - {}ms - Error: {s}{s}\n", .{ colors.RED, request.method, request.url, result.status_code, result.duration_ms, msg, colors.RESET });
                } else {
                    customPrint(maybe_log_file, "{s}❌ {s} {s} - Status: {} - {}ms{s}\n", .{ colors.RED, request.method, request.url, result.status_code, result.duration_ms, colors.RESET });
                }
            }

            if (verbose) {
                customPrint(maybe_log_file, "\n{s}📥 Response Details:{s}\n", .{ colors.BLUE, colors.RESET });
                customPrint(maybe_log_file, "Status: {}\n", .{result.status_code});
                customPrint(maybe_log_file, "Duration: {}ms\n", .{result.duration_ms});

                if (result.response_headers) |headers| {
                    customPrint(maybe_log_file, "Headers:\n", .{});
                    for (headers) |header| {
                        customPrint(maybe_log_file, "  {s}: {s}\n", .{ header.name, header.value });
                    }
                }

                if (result.response_body) |body| {
                    customPrint(maybe_log_file, "Body:\n{s}\n", .{body});
                }
                customPrint(maybe_log_file, "{s}\n", .{"-" ** 30});
            }
        }

        customPrint(maybe_log_file, "\n{s}\n", .{"=" ** 50});
        customPrint(maybe_log_file, "File Summary: {s}{}{s}/{} requests succeeded\n\n", .{ if (success_count == request_count) colors.GREEN else if (success_count > 0) colors.YELLOW else colors.RED, success_count, colors.RESET, request_count });

        total_success_count += success_count;
        total_request_count += request_count;
    }

    if (files_processed > 1) {
        customPrint(maybe_log_file, "{s}🎯 Overall Summary:{s}\n", .{ colors.BLUE, colors.RESET });
        customPrint(maybe_log_file, "Files processed: {}\n", .{files_processed});
        customPrint(maybe_log_file, "Total requests: {s}{}{s}/{}\n", .{ if (total_success_count == total_request_count) colors.GREEN else if (total_success_count > 0) colors.YELLOW else colors.RED, total_success_count, colors.RESET, total_request_count });
    }
}
