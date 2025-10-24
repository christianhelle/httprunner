const std = @import("std");
const print = std.debug.print;
const Allocator = std.mem.Allocator;

pub const Log = struct {
    log_file: ?std.fs.File,

    pub fn init(allocator: Allocator, base_filename: ?[]const u8) !Log {
        var log_file: ?std.fs.File = null;
        if (base_filename) |filename| {
            log_file = try createLogFile(allocator, filename);
        } else {
            log_file = null;
        }
        return Log{ .log_file = log_file };
    }

    pub fn deinit(self: *Log) void {
        if (self.log_file) |file| {
            file.close();
        }
    }

    pub fn write(self: *Log, comptime fmt: []const u8, args: anytype) void {
        print(fmt, args);
        if (self.log_file) |file| {
            const message = std.fmt.allocPrint(std.heap.page_allocator, fmt, args) catch return;
            defer std.heap.page_allocator.free(message);
            file.writeAll(message) catch |err| {
                print("Error writing to log file: {}\n", .{err});
            };
        }
    }
};

pub fn createLogFile(allocator: Allocator, base_filename: []const u8) !?std.fs.File {
    if (base_filename.len == 0) return null;
    const timestamp = @as(u64, @intCast(std.time.timestamp()));
    const log_filename = try std.fmt.allocPrint(allocator, "{s}_{d}.log", .{ base_filename, timestamp });
    defer allocator.free(log_filename);
    return try std.fs.cwd().createFile(log_filename, .{});
}
