const std = @import("std");
const print = std.debug.print;
const Allocator = std.mem.Allocator;

const colors = @import("colors.zig");

pub fn discoverHttpFiles(allocator: Allocator, dir_path: []const u8, files: *std.ArrayList([]const u8)) !void {
    var dir = std.fs.cwd().openDir(dir_path, .{ .iterate = true }) catch |err| {
        switch (err) {
            error.FileNotFound, error.NotDir => return,
            else => return err,
        }
    };
    defer dir.close();

    var iterator = dir.iterate();
    while (try iterator.next()) |entry| {
        switch (entry.kind) {
            .file => {
                if (std.mem.endsWith(u8, entry.name, ".http")) {
                    const full_path = try std.fs.path.join(allocator, &[_][]const u8{ dir_path, entry.name });
                    try files.append(full_path);
                }
            },
            .directory => {
                if (!std.mem.eql(u8, entry.name, ".") and !std.mem.eql(u8, entry.name, "..")) {
                    const sub_dir = try std.fs.path.join(allocator, &[_][]const u8{ dir_path, entry.name });
                    defer allocator.free(sub_dir);
                    try discoverHttpFiles(allocator, sub_dir, files);
                }
            },
            else => {},
        }
    }
}

pub fn runDiscoveryMode(allocator: Allocator, files: *std.ArrayList([]const u8)) !bool {
    print("{s}🔍 Discovering .http files recursively...{s}\n", .{ colors.BLUE, colors.RESET });

    try discoverHttpFiles(allocator, ".", files);

    if (files.items.len == 0) {
        print("{s}⚠️  No .http files found in current directory and subdirectories{s}\n", .{ colors.YELLOW, colors.RESET });
        return false;
    }

    print("Found {} .http file(s):\n", .{files.items.len});
    for (files.items) |file_path| {
        print("  📄 {s}\n", .{file_path});
    }
    print("\n", .{});

    return true;
}
