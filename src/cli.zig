const std = @import("std");
const print = std.debug.print;
const Allocator = std.mem.Allocator;

const colors = @import("colors.zig");
const version_info = @import("version_info.zig");

pub const CliOptions = struct {
    discover_mode: bool,
    verbose: bool,
    show_version: bool,
    log_file: ?[]const u8,
    environment: ?[]const u8,
    files: []const []const u8,
    allocator: ?Allocator,

    pub fn deinit(self: *CliOptions) void {
        if (self.allocator != null and self.files.len > 0) {
            self.allocator.?.free(self.files);
        }
    }
    pub fn parse(allocator: Allocator, args: []const []const u8) !CliOptions {
        // Check for help flag first
        if (containsFlag(args, "--help") or containsFlag(args, "-h")) {
            showUsage();
            return error.InvalidArguments;
        }

        // Check for version flag first
        if (containsFlag(args, "--version") or containsFlag(args, "-v")) {
            return CliOptions{
                .discover_mode = false,
                .verbose = false,
                .show_version = true,
                .log_file = null,
                .environment = null,
                .files = &[_][]const u8{},
                .allocator = null,
            };
        }

        if (args.len < 2) {
            showUsage();
            return error.InvalidArguments;
        }

        var discover_mode = false;
        var verbose = false;
        var log_file: ?[]const u8 = null;
        var environment: ?[]const u8 = null;

        if (containsFlag(args, "--discover")) {
            discover_mode = true;

            verbose = containsFlag(args, "--verbose");
            log_file = getLogFilename(args);
            environment = getEnvironment(args);
            return CliOptions{
                .discover_mode = true,
                .verbose = verbose,
                .show_version = false,
                .log_file = log_file,
                .environment = environment,
                .files = &[_][]const u8{},
                .allocator = null,
            };
        }

        verbose = containsFlag(args, "--verbose");
        log_file = getLogFilename(args);
        environment = getEnvironment(args);

        var files_list = std.ArrayList([]const u8).init(allocator);
        defer files_list.deinit();
        var i: usize = 1;
        while (i < args.len) : (i += 1) {
            const arg = args[i];
            if (std.mem.eql(u8, arg, "--discover") or
                std.mem.eql(u8, arg, "--verbose") or
                std.mem.eql(u8, arg, "--version") or
                std.mem.eql(u8, arg, "--help") or
                std.mem.eql(u8, arg, "-v") or
                std.mem.eql(u8, arg, "-h"))
            {
                continue;
            } else if (std.mem.eql(u8, arg, "--log")) {
                if (i + 1 < args.len and !std.mem.startsWith(u8, args[i + 1], "--")) {
                    i += 1;
                }
                continue;
            } else if (std.mem.eql(u8, arg, "--env")) {
                if (i + 1 >= args.len or std.mem.startsWith(u8, args[i + 1], "--")) {
                    return error.InvalidArguments;
                }
                i += 1;
                continue;
            } else {
                try files_list.append(arg);
            }
        }

        if (files_list.items.len == 0) {
            showUsage();
            return error.InvalidArguments;
        }

        var files_owned = try allocator.alloc([]const u8, files_list.items.len);
        for (files_list.items, 0..) |file, idx| {
            files_owned[idx] = file;
        }
        return CliOptions{
            .discover_mode = discover_mode,
            .verbose = verbose,
            .show_version = false,
            .log_file = log_file,
            .environment = environment,
            .files = files_owned,
            .allocator = allocator,
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
            if (i + 1 >= args.len or std.mem.startsWith(u8, args[i + 1], "--")) {
                return "log";
            } else {
                return args[i + 1];
            }
        }
    }
    return null;
}

fn getEnvironment(args: []const []const u8) ?[]const u8 {
    var i: usize = 1;
    while (i < args.len) : (i += 1) {
        if (std.mem.eql(u8, args[i], "--env")) {
            if (i + 1 >= args.len or std.mem.startsWith(u8, args[i + 1], "--")) {
                return null;
            } else {
                return args[i + 1];
            }
        }
    }
    return null;
}

pub fn showUsage() void {
    print("{s}Usage:{s}\n", .{ colors.BLUE, colors.RESET });
    print("  httprunner <http-file> [http-file2] [...] [--verbose] [--log [filename]] [--env <environment>]\n", .{});
    print("  httprunner [--verbose] [--log [filename]] [--env <environment>] --discover\n", .{});
    print("  httprunner --version | -v\n", .{});
    print("  httprunner --help | -h\n", .{});
    print("\n{s}Arguments:{s}\n", .{ colors.BLUE, colors.RESET });
    print("  <http-file>    One or more .http files to process\n", .{});
    print("  --discover     Recursively discover and process all .http files from current directory\n", .{});
    print("  --verbose      Show detailed HTTP request and response information\n", .{});
    print("  --log [file]   Log output to a file (defaults to 'log' if no filename is specified)\n", .{});
    print("  --env <env>    Specify environment name to load variables from http-client.env.json\n", .{});
    print("  --version, -v  Show version information\n", .{});
    print("  --help, -h     Show this help message\n", .{});
}

pub fn showVersion() void {
    print("{s}httprunner{s} version {s}{s}{s}\n", .{ colors.BLUE, colors.RESET, colors.GREEN, version_info.VERSION, colors.RESET });
    print("Git tag: {s}{s}{s}\n", .{ colors.YELLOW, version_info.GIT_TAG, colors.RESET });
    print("Git commit: {s}{s}{s}\n", .{ colors.YELLOW, version_info.GIT_COMMIT, colors.RESET });
    print("Build date: {s}{s}{s}\n", .{ colors.YELLOW, version_info.BUILD_DATE, colors.RESET });
}
