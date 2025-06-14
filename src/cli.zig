const std = @import("std");
const print = std.debug.print;
const Allocator = std.mem.Allocator;

const colors = @import("colors.zig");

pub const CliOptions = struct {
    discover_mode: bool,
    verbose: bool,
    log_file: ?[]const u8,
    files: []const []const u8,
    pub fn parse(_: Allocator, args: []const []const u8) !CliOptions {
        if (args.len < 2) {
            showUsage();
            return error.InvalidArguments;
        }

        var discover_mode = false;
        var verbose = false;
        var log_file: ?[]const u8 = null;
        var file_start_index: usize = 1;

        // Parse flags
        var i: usize = 1;
        while (i < args.len) : (i += 1) {
            const arg = args[i];
            if (std.mem.eql(u8, arg, "--discover")) {
                discover_mode = true;
            } else if (std.mem.eql(u8, arg, "--verbose")) {
                verbose = true;
            } else if (std.mem.eql(u8, arg, "--log")) {
                // Check if there's a value after --log
                if (i + 1 >= args.len) {
                    print("Error: --log requires a filename argument\n", .{});
                    showUsage();
                    return error.InvalidArguments;
                }
                i += 1;
                log_file = args[i];
            } else {
                // First non-flag argument, stop parsing flags
                file_start_index = i;
                break;
            }
        }

        if (discover_mode) {
            return CliOptions{
                .discover_mode = true,
                .verbose = verbose,
                .log_file = log_file,
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
                .log_file = log_file,
                .files = args[file_start_index..],
            };
        }
    }
};

pub fn showUsage() void {
    print("{s}Usage:{s}\n", .{ colors.BLUE, colors.RESET });
    print("  httprunner [--verbose] [--log <filename>] <http-file> [http-file2] [...]\n", .{});
    print("  httprunner [--verbose] [--log <filename>] --discover\n", .{});
    print("\n{s}Arguments:{s}\n", .{ colors.BLUE, colors.RESET });
    print("  <http-file>    One or more .http files to process\n", .{});
    print("  --discover     Recursively discover and process all .http files from current directory\n", .{});
    print("  --verbose      Show detailed HTTP request and response information\n", .{});
    print("  --log <file>   Log output to the specified file with timestamp appended to prevent overwriting\n", .{});
}
