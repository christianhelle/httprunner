const std = @import("std");
const print = std.debug.print;
const Allocator = std.mem.Allocator;

// Import our modules
const colors = @import("colors.zig");
const types = @import("types.zig");
const parser = @import("parser.zig");
const executor = @import("executor.zig");

// Type aliases for convenience
const HttpRequest = types.HttpRequest;
const HttpResult = types.HttpResult;

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);
    if (args.len < 2) {
        print("{s}❌ Usage: {s} <http-file>{s}\n", .{ colors.RED, args[0], colors.RESET });
        return;
    }

    const http_file = args[1];
    print("{s}🚀 HTTP Runner - Processing file: {s}{s}\n", .{ colors.BLUE, http_file, colors.RESET });
    print("{s}\n", .{"=" ** 50});

    const requests = parser.parseHttpFile(allocator, http_file) catch |err| {
        switch (err) {
            error.FileNotFound => {
                print("{s}❌ Error: File '{s}' not found{s}\n", .{ colors.RED, http_file, colors.RESET });
                return;
            },
            else => {
                print("{s}❌ Error parsing file: {}{s}\n", .{ colors.RED, err, colors.RESET });
                return;
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
        return;
    }

    print("Found {} HTTP request(s)\n\n", .{requests.items.len});

    var success_count: u32 = 0;
    var total_count: u32 = 0;

    for (requests.items) |request| {
        total_count += 1;
        const result = executor.executeHttpRequest(allocator, request) catch |err| {
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
    print("Summary: {s}{}{s}/{} requests succeeded\n", .{ if (success_count == total_count) colors.GREEN else if (success_count > 0) colors.YELLOW else colors.RED, success_count, colors.RESET, total_count });
}

test "simple test" {
    var list = std.ArrayList(i32).init(std.testing.allocator);
    defer list.deinit(); // try commenting this out and see if zig detects the memory leak!
    try list.append(42);
    try std.testing.expectEqual(@as(i32, 42), list.pop());
}
