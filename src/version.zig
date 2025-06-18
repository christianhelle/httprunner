const std = @import("std");

pub fn getVersionInfo(allocator: std.mem.Allocator) !struct {
    version: []const u8,
    git_tag: []const u8,
    git_commit: []const u8,
    build_date: []const u8,
} {
    const result = std.process.Child.run(.{
        .allocator = allocator,
        .argv = &[_][]const u8{ "git", "describe", "--tags", "--always", "--dirty" },
    }) catch {
        // Fallback if git is not available
        return .{
            .version = "unknown",
            .git_tag = "unknown",
            .git_commit = "unknown",
            .build_date = "unknown",
        };
    };
    defer allocator.free(result.stdout);
    defer allocator.free(result.stderr);

    const git_info = std.mem.trim(u8, result.stdout, " \t\n\r");

    // Get the current commit hash
    const commit_result = std.process.Child.run(.{
        .allocator = allocator,
        .argv = &[_][]const u8{ "git", "rev-parse", "--short", "HEAD" },
    }) catch {
        return .{
            .version = try allocator.dupe(u8, git_info),
            .git_tag = try allocator.dupe(u8, git_info),
            .git_commit = "unknown",
            .build_date = "unknown",
        };
    };
    defer allocator.free(commit_result.stdout);
    defer allocator.free(commit_result.stderr);

    const commit_hash = std.mem.trim(u8, commit_result.stdout, " \t\n\r");

    // Get build date
    const timestamp = std.time.timestamp();
    const epoch_seconds = @as(u64, @intCast(timestamp));
    const build_date = try std.fmt.allocPrint(allocator, "{d}", .{epoch_seconds});

    return .{
        .version = try allocator.dupe(u8, git_info),
        .git_tag = try allocator.dupe(u8, git_info),
        .git_commit = try allocator.dupe(u8, commit_hash),
        .build_date = build_date,
    };
}
