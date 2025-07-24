const std = @import("std");

pub fn build(b: *std.Build) void {
    // Standard target options allows the person running `zig build` to choose
    // what target to build for. Here we do not override the defaults, which
    // means any target is allowed, and the default is native.
    const target = b.standardTargetOptions(.{});

    // Standard optimization options allow the person running `zig build` to select
    // between Debug, ReleaseSafe, ReleaseFast, and ReleaseSmall.
    const optimize = b.standardOptimizeOption(.{});

    // Generate version information at build time
    const version_step = generateVersionStep(b);

    const exe = b.addExecutable(.{
        .name = "httprunner",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    // Generate version info before building
    exe.step.dependOn(version_step);

    // This declares intent for the executable to be installed into the
    // standard location when the user invokes the "install" step (the default
    // step when running `zig build`).
    b.installArtifact(exe);

    // This *creates* a Run step in the build graph, to be executed when another
    // step is evaluated that depends on it. The next line below will establish
    // such a dependency.
    const run_cmd = b.addRunArtifact(exe);

    // By making the run step depend on the install step, it will be run from the
    // installation directory rather than directly from within the cache directory.
    // This is not necessary, however, if the application depends on other installed
    // files, this ensures they will be present and in the expected location.
    run_cmd.step.dependOn(b.getInstallStep());

    // This allows the user to pass arguments to the application in the build
    // command itself, like this: `zig build run -- arg1 arg2 etc`
    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    // This creates a build step. It will be visible in the `zig build --help` menu,
    // and can be selected like this: `zig build run`
    // This will evaluate the `run` step rather than the default, which is "install".
    const run_step = b.step("run", "Run the app");
    run_step.dependOn(&run_cmd.step);

    // Creates a step for unit testing.
    const unit_tests = b.addTest(.{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    const run_unit_tests = b.addRunArtifact(unit_tests);

    // Similar to creating the run step earlier, this exposes a `test` step to
    // the `zig build --help` menu, providing a way for the user to request
    // running the unit tests.
    const test_step = b.step("test", "Run unit tests");
    test_step.dependOn(&run_unit_tests.step);
}

fn generateVersionStep(b: *std.Build) *std.Build.Step {
    const step = b.allocator.create(std.Build.Step) catch @panic("OOM");
    step.* = std.Build.Step.init(.{
        .id = .custom,
        .name = "generate-version-info",
        .owner = b,
        .makeFn = makeVersionInfo,
    });
    return step;
}

fn makeVersionInfo(step: *std.Build.Step, options: std.Build.Step.MakeOptions) !void {
    _ = options;
    const b = step.owner;
    const allocator = b.allocator;

    // Get git information at build time (simplified for reliability)
    const git_tag = getGitOutput(allocator, &.{ "git", "describe", "--tags", "--abbrev=0" }) orelse "unknown";
    const git_commit = getGitOutput(allocator, &.{ "git", "rev-parse", "--short", "HEAD" }) orelse "unknown";

    // Parse version from git tag (remove 'v' prefix if present)
    const version = if (std.mem.startsWith(u8, git_tag, "v"))
        git_tag[1..]
    else
        git_tag;

    // Get current timestamp and format it accurately using std.time
    const timestamp = std.time.timestamp();
    const epoch_seconds = std.time.epoch.EpochSeconds{ .secs = @as(u64, @intCast(timestamp)) };
    const epoch_day = epoch_seconds.getEpochDay();
    const day_seconds = epoch_seconds.getDaySeconds();
    const year_day = epoch_day.calculateYearDay();
    const month_day = year_day.calculateMonthDay();

    var date_buf: [64]u8 = undefined;
    const build_date = std.fmt.bufPrint(&date_buf, "{d}-{d:0>2}-{d:0>2} {d:0>2}:{d:0>2}:{d:0>2} UTC", .{ year_day.year, month_day.month.numeric(), month_day.day_index + 1, day_seconds.getHoursIntoDay(), day_seconds.getMinutesIntoHour(), day_seconds.getSecondsIntoMinute() }) catch "unknown";

    // Generate version_info.zig content
    const content = std.fmt.allocPrint(allocator,
        \\// This file is auto-generated at build time
        \\pub const VERSION = "{s}";
        \\pub const GIT_TAG = "{s}";
        \\pub const GIT_COMMIT = "{s}";
        \\pub const BUILD_DATE = "{s}";
        \\
    , .{ version, git_tag, git_commit, build_date }) catch @panic("OOM");

    // Write directly to the src directory
    const file_path = "src/version_info.zig";
    std.fs.cwd().writeFile(.{ .sub_path = file_path, .data = content }) catch |err| {
        std.log.err("Failed to write version_info.zig: {}", .{err});
        return;
    };

    std.log.info("Generated version info: {s} ({s} - {s})", .{ version, git_tag, git_commit });
}

fn getGitOutput(allocator: std.mem.Allocator, argv: []const []const u8) ?[]const u8 {
    var child = std.process.Child.init(argv, allocator);
    child.stdout_behavior = .Pipe;
    child.stderr_behavior = .Pipe;

    child.spawn() catch return null;

    const stdout = child.stdout.?.readToEndAlloc(allocator, 1024) catch return null;
    const stderr = child.stderr.?.readToEndAlloc(allocator, 1024) catch {
        allocator.free(stdout);
        return null;
    };
    defer allocator.free(stderr);

    const term = child.wait() catch {
        allocator.free(stdout);
        return null;
    };

    switch (term) {
        .Exited => |code| {
            if (code == 0) {
                return std.mem.trim(u8, stdout, " \t\n\r");
            } else {
                allocator.free(stdout);
                return null;
            }
        },
        else => {
            allocator.free(stdout);
            return null;
        },
    }
}
