const std = @import("std");
const Allocator = std.mem.Allocator;

const cli = @import("cli.zig");
const colors = @import("colors.zig");
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
            error.InvalidArguments => {
                cli.showUsage();
                return;
            },
            else => return err,
        }
    };
    defer options.deinit();

    if (options.show_version) {
        cli.showVersion();
        return;
    }

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
            try processHttpFiles(allocator, discovered_files.items, options.verbose, options.log_file, options.environment);
        }
    } else {
        try processHttpFiles(allocator, options.files, options.verbose, options.log_file, options.environment);
    }
}

fn processHttpFiles(allocator: Allocator, files: []const []const u8, verbose: bool, log_filename: ?[]const u8, environment: ?[]const u8) !void {
    if (try processor.processHttpFiles(allocator, files, verbose, log_filename, environment)) {
        std.debug.print("{s}✅ All discovered files processed successfully{s}\n", .{ colors.GREEN, colors.RESET });
    } else {
        std.debug.print("{s}❌ Some discovered files failed to process{s}\n", .{ colors.RED, colors.RESET });
        std.process.exit(1);
    }
}
