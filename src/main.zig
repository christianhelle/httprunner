const std = @import("std");
const print = std.debug.print;
const Allocator = std.mem.Allocator;

const colors = @import("colors.zig");
const types = @import("types.zig");
const parser = @import("parser.zig");
const runner = @import("runner.zig");

const HttpRequest = types.HttpRequest;
const HttpResult = types.HttpResult;

fn discoverHttpFiles(allocator: Allocator, dir_path: []const u8, files: *std.ArrayList([]const u8)) !void {
    var dir = std.fs.cwd().openDir(dir_path, .{ .iterate = true }) catch |err| {
        switch (err) {
            error.FileNotFound, error.NotDir => return,
            else => return err,
        }
    };
    defer dir.close();

    var iterator = dir.iterate();
    while (try iterator.next()) |entry| {
        switch (entry.kind) {
            .file => {
                if (std.mem.endsWith(u8, entry.name, ".http")) {
                    const full_path = try std.fs.path.join(allocator, &[_][]const u8{ dir_path, entry.name });
                    try files.append(full_path);
                }
            },
            .directory => {
                if (!std.mem.eql(u8, entry.name, ".") and !std.mem.eql(u8, entry.name, "..")) {
                    const sub_dir = try std.fs.path.join(allocator, &[_][]const u8{ dir_path, entry.name });
                    defer allocator.free(sub_dir);
                    try discoverHttpFiles(allocator, sub_dir, files);
                }
            },
            else => {},
        }
    }
}

fn showUsage(program_name: []const u8) void {
    print("{s}Usage:{s}\n", .{ colors.BLUE, colors.RESET });
    print("  {s} <http-file> [http-file2] [...]\n", .{program_name});
    print("  {s} --discover\n", .{program_name});
    print("\n{s}Arguments:{s}\n", .{ colors.BLUE, colors.RESET });
    print("  <http-file>    One or more .http files to process\n", .{});
    print("  --discover     Recursively discover and process all .http files from current directory\n", .{});
}

fn processHttpFiles(allocator: Allocator, files: []const []const u8) !void {
    var total_success_count: u32 = 0;
    var total_request_count: u32 = 0;
    var files_processed: u32 = 0;

    for (files) |http_file| {
        print("{s}🚀 HTTP File Runner - Processing file: {s}{s}\n", .{ colors.BLUE, http_file, colors.RESET });
        print("{s}\n", .{"=" ** 50});

        const requests = parser.parseHttpFile(allocator, http_file) catch |err| {
            switch (err) {
                error.FileNotFound => {
                    print("{s}❌ Error: File '{s}' not found{s}\n", .{ colors.RED, http_file, colors.RESET });
                    continue;
                },
                else => {
                    print("{s}❌ Error parsing file: {}{s}\n", .{ colors.RED, err, colors.RESET });
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
            print("{s}⚠️  No HTTP requests found in file{s}\n", .{ colors.YELLOW, colors.RESET });
            continue;
        }

        print("Found {} HTTP request(s)\n\n", .{requests.items.len});
        files_processed += 1;

        var success_count: u32 = 0;
        var request_count: u32 = 0;

        for (requests.items) |request| {
            request_count += 1;
            const result = runner.executeHttpRequest(allocator, request) catch |err| {
                print("{s}❌ {s} {s} - Error: {}{s}\n", .{ colors.RED, request.method, request.url, err, colors.RESET });
                continue;
            };
            if (result.success) {
                success_count += 1;
                print("{s}✅ {s} {s} - Status: {} - {}ms{s}\n", .{ colors.GREEN, request.method, request.url, result.status_code, result.duration_ms, colors.RESET });
            } else {
                if (result.error_message) |msg| {
                    print("{s}❌ {s} {s} - Status: {} - {}ms - Error: {s}{s}\n", .{ colors.RED, request.method, request.url, result.status_code, result.duration_ms, msg, colors.RESET });
                } else {
                    print("{s}❌ {s} {s} - Status: {} - {}ms{s}\n", .{ colors.RED, request.method, request.url, result.status_code, result.duration_ms, colors.RESET });
                }
            }
        }

        print("\n{s}\n", .{"=" ** 50});
        print("File Summary: {s}{}{s}/{} requests succeeded\n\n", .{ if (success_count == request_count) colors.GREEN else if (success_count > 0) colors.YELLOW else colors.RED, success_count, colors.RESET, request_count });

        total_success_count += success_count;
        total_request_count += request_count;
    }

    if (files_processed > 1) {
        print("{s}🎯 Overall Summary:{s}\n", .{ colors.BLUE, colors.RESET });
        print("Files processed: {}\n", .{files_processed});
        print("Total requests: {s}{}{s}/{}\n", .{ if (total_success_count == total_request_count) colors.GREEN else if (total_success_count > 0) colors.YELLOW else colors.RED, total_success_count, colors.RESET, total_request_count });
    }
}

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 2) {
        showUsage(args[0]);
        return;
    }

    // Check for --discover flag
    if (std.mem.eql(u8, args[1], "--discover")) {
        print("{s}🔍 Discovering .http files recursively...{s}\n", .{ colors.BLUE, colors.RESET });

        var discovered_files = std.ArrayList([]const u8).init(allocator);
        defer {
            for (discovered_files.items) |file_path| {
                allocator.free(file_path);
            }
            discovered_files.deinit();
        }

        try discoverHttpFiles(allocator, ".", &discovered_files);

        if (discovered_files.items.len == 0) {
            print("{s}⚠️  No .http files found in current directory and subdirectories{s}\n", .{ colors.YELLOW, colors.RESET });
            return;
        }

        print("Found {} .http file(s):\n", .{discovered_files.items.len});
        for (discovered_files.items) |file_path| {
            print("  📄 {s}\n", .{file_path});
        }
        print("\n", .{});

        try processHttpFiles(allocator, discovered_files.items);
    } else {
        // Process specified files
        const http_files = args[1..];
        try processHttpFiles(allocator, http_files);
    }
}

test "simple test" {
    var list = std.ArrayList(i32).init(std.testing.allocator);
    defer list.deinit();
    try list.append(42);
    try std.testing.expectEqual(@as(i32, 42), list.pop());
}
