const std = @import("std");
const print = std.debug.print;
const Allocator = std.mem.Allocator;

const colors = @import("colors.zig");

pub const CliOptions = struct {
    discover_mode: bool,
    verbose: bool,
    files: []const []const u8,
    pub fn parse(_: Allocator, args: []const []const u8) !CliOptions {
        if (args.len < 2) {
            showUsage();
            return error.InvalidArguments;
        }

        var discover_mode = false;
        var verbose = false;
        var file_start_index: usize = 1;

        // Parse flags
        for (args[1..], 0..) |arg, i| {
            if (std.mem.eql(u8, arg, "--discover")) {
                discover_mode = true;
                file_start_index = i + 2; // +2 because we're in args[1..] and need to account for that
            } else if (std.mem.eql(u8, arg, "--verbose")) {
                verbose = true;
                file_start_index = i + 2; // +2 because we're in args[1..] and need to account for that
            } else {
                // First non-flag argument, stop parsing flags
                file_start_index = i + 1; // +1 because we're in args[1..]
                break;
            }
        }

        if (discover_mode) {
            return CliOptions{
                .discover_mode = true,
                .verbose = verbose,
                .files = &[_][]const u8{},
            };
        } else if (file_start_index >= args.len) {
            // No files provided after flags
            showUsage();
            return error.InvalidArguments;
        } else {
            return CliOptions{
                .discover_mode = false,
                .verbose = verbose,
                .files = args[file_start_index..],
            };
        }
    }
};

pub fn showUsage() void {
    print("{s}Usage:{s}\n", .{ colors.BLUE, colors.RESET });
    print("  httprunner [--verbose] <http-file> [http-file2] [...]\n", .{});
    print("  httprunner [--verbose] --discover\n", .{});
    print("\n{s}Arguments:{s}\n", .{ colors.BLUE, colors.RESET });
    print("  <http-file>    One or more .http files to process\n", .{});
    print("  --discover     Recursively discover and process all .http files from current directory\n", .{});
    print("  --verbose      Show detailed HTTP request and response information\n", .{});
}
