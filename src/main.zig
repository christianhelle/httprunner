const std = @import("std");
const Allocator = std.mem.Allocator;

const cli = @import("cli.zig");
const colors = @import("colors.zig");
const discovery = @import("discovery.zig");
const processor = @import("processor.zig");
const upgrade = @import("upgrade.zig");
const curl = @import("curl.zig");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Initialize libcurl
    try curl.init();
    defer curl.deinit();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);
    var options = cli.CliOptions.parse(allocator, args) catch |err| {
        switch (err) {
            error.InvalidArguments => {
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

    if (options.upgrade_mode) {
        try upgrade.runUpgrade();
        return;
    }

    if (options.discover_mode) {
        var discovered_files = std.ArrayList([]const u8).initCapacity(allocator, 0) catch @panic("OOM");
        defer {
            for (discovered_files.items) |file_path| {
                allocator.free(file_path);
            }
            discovered_files.deinit(allocator);
        }
        const found_files = try discovery.runDiscoveryMode(allocator, &discovered_files);
        if (found_files) {
            try processHttpFiles(allocator, discovered_files.items, options);
        }
    } else {
        try processHttpFiles(allocator, options.files, options);
    }
}

fn processHttpFiles(allocator: Allocator, files: []const []const u8, options: cli.CliOptions) !void {
    const result = try processor.processHttpFiles(allocator, files, options.verbose, options.log_file, options.environment, options.insecure);
    if (result) {
        std.debug.print("{s}✅ All discovered files processed successfully{s}\n", .{ colors.GREEN, colors.RESET });
    } else {
        std.debug.print("{s}❌ Some discovered files failed to process{s}\n", .{ colors.RED, colors.RESET });
    }

    cli.showDonationBanner();

    if (result) {
        std.process.exit(1);
    }
}
