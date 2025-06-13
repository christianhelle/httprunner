const std = @import("std");
const print = std.debug.print;
const Allocator = std.mem.Allocator;

const colors = @import("colors.zig");

pub const CliOptions = struct {
    discover_mode: bool,
    files: []const []const u8,

    pub fn parse(_: Allocator, args: []const []const u8) !CliOptions {
        if (args.len < 2) {
            showUsage(args[0]);
            return error.InvalidArguments;
        }

        if (std.mem.eql(u8, args[1], "--discover")) {
            return CliOptions{
                .discover_mode = true,
                .files = &[_][]const u8{},
            };
        } else {
            return CliOptions{
                .discover_mode = false,
                .files = args[1..],
            };
        }
    }
};

pub fn showUsage(program_name: []const u8) void {
    print("{s}Usage:{s}\n", .{ colors.BLUE, colors.RESET });
    print("  {s} <http-file> [http-file2] [...]\n", .{program_name});
    print("  {s} --discover\n", .{program_name});
    print("\n{s}Arguments:{s}\n", .{ colors.BLUE, colors.RESET });
    print("  <http-file>    One or more .http files to process\n", .{});
    print("  --discover     Recursively discover and process all .http files from current directory\n", .{});
}
