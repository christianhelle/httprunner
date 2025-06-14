const std = @import("std");
const print = std.debug.print;
const Allocator = std.mem.Allocator;

const colors = @import("colors.zig");

pub const CliOptions = struct {
    discover_mode: bool,
    verbose: bool,
    log_file: ?[]const u8,
    files: []const []const u8,
    allocator: ?Allocator, // Store allocator for later deallocation

    pub fn deinit(self: *CliOptions) void {
        // Only free memory if we have an allocator and own allocated files
        if (self.allocator != null and self.files.len > 0) {
            // Free the array itself
            self.allocator.?.free(self.files);
        }
    }

    pub fn parse(allocator: Allocator, args: []const []const u8) !CliOptions {
        if (args.len < 2) {
            showUsage();
            return error.InvalidArguments;
        }

        var discover_mode = false;
        var verbose = false;
        var log_file: ?[]const u8 = null;

        // For discovery mode, don't need files
        if (containsFlag(args, "--discover")) {
            discover_mode = true;

            // Check for other flags
            verbose = containsFlag(args, "--verbose");
            log_file = getLogFilename(args);

            return CliOptions{
                .discover_mode = true,
                .verbose = verbose,
                .log_file = log_file,
                .files = &[_][]const u8{},
                .allocator = null, // Empty array is static, no need to free
            };
        }

        // Not discovery mode, collect files and flags
        verbose = containsFlag(args, "--verbose");
        log_file = getLogFilename(args);

        // Files are any arguments not starting with -- and not following --log
        var files_list = std.ArrayList([]const u8).init(allocator);
        defer files_list.deinit();
        // Collect all arguments that aren't flags or flag parameters
        var i: usize = 1;
        while (i < args.len) : (i += 1) {
            const arg = args[i];

            // Skip flags without parameters
            if (std.mem.eql(u8, arg, "--discover") or
                std.mem.eql(u8, arg, "--verbose"))
            {
                continue;
            }
            // Skip --log and its parameter (if it has one)
            else if (std.mem.eql(u8, arg, "--log")) {
                // Skip the next argument only if it's not a flag (starts with --)
                // and we're not at the end of args
                if (i + 1 < args.len and !std.mem.startsWith(u8, args[i + 1], "--")) {
                    i += 1; // Skip the filename parameter
                }
                continue;
            }
            // Must be a file
            else {
                try files_list.append(arg);
            }
        }

        // Check if we have files
        if (files_list.items.len == 0) {
            // No files provided
            showUsage();
            return error.InvalidArguments;
        }

        // Copy the files to a new slice owned by the caller
        var files_owned = try allocator.alloc([]const u8, files_list.items.len);
        for (files_list.items, 0..) |file, idx| {
            files_owned[idx] = file;
        }

        return CliOptions{
            .discover_mode = discover_mode,
            .verbose = verbose,
            .log_file = log_file,
            .files = files_owned,
            .allocator = allocator, // Store allocator for later cleanup
        };
    }
};

fn containsFlag(args: []const []const u8, flag_name: []const u8) bool {
    for (args) |arg| {
        if (std.mem.eql(u8, arg, flag_name)) {
            return true;
        }
    }
    return false;
}

fn getLogFilename(args: []const []const u8) ?[]const u8 {
    var i: usize = 1;
    while (i < args.len) : (i += 1) {
        if (std.mem.eql(u8, args[i], "--log")) {
            // Default to "log" as the filename
            if (i + 1 >= args.len or std.mem.startsWith(u8, args[i + 1], "--")) {
                return "log";
            } else {
                return args[i + 1];
            }
        }
    }
    return null; // No log flag found
}

pub fn showUsage() void {
    print("{s}Usage:{s}\n", .{ colors.BLUE, colors.RESET });
    print("  httprunner [--verbose] [--log [filename]] <http-file> [http-file2] [...]\n", .{});
    print("  httprunner [--verbose] [--log [filename]] --discover\n", .{});
    print("\n{s}Arguments:{s}\n", .{ colors.BLUE, colors.RESET });
    print("  <http-file>    One or more .http files to process\n", .{});
    print("  --discover     Recursively discover and process all .http files from current directory\n", .{});
    print("  --verbose      Show detailed HTTP request and response information\n", .{});
    print("  --log [file]   Log output to a file (defaults to 'log' if no filename is specified)\n", .{});
}
