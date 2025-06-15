const std = @import("std");
const Allocator = std.mem.Allocator;

const cli = @import("cli.zig");
const discovery = @import("discovery.zig");
const processor = @import("processor.zig");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    var options = cli.CliOptions.parse(allocator, args) catch |err| {
        switch (err) {
            error.InvalidArguments => return,
            else => return err,
        }
    };
    defer options.deinit();

    if (options.discover_mode) {
        var discovered_files = std.ArrayList([]const u8).init(allocator);
        defer {
            for (discovered_files.items) |file_path| {
                allocator.free(file_path);
            }
            discovered_files.deinit();
        }
        const found_files = try discovery.runDiscoveryMode(allocator, &discovered_files);
        if (found_files) {
            try processor.processHttpFiles(allocator, discovered_files.items, options.verbose, options.log_file);
        }
    } else {
        try processor.processHttpFiles(allocator, options.files, options.verbose, options.log_file);
    }
}
