const std = @import("std");
const builtin = @import("builtin");
const print = std.debug.print;

const colors = @import("colors.zig");

pub fn runUpgrade() !void {
    print("{s}🚀 Upgrading httprunner to the latest version...{s}\n", .{ colors.BLUE, colors.RESET });

    const command = switch (builtin.os.tag) {
        .windows => "irm https://christianhelle.com/httprunner/install.ps1 | iex",
        .linux, .macos => "curl -fsSL https://christianhelle.com/httprunner/install | bash",
        else => {
            print("{s}❌ Upgrade is not supported on this platform{s}\n", .{ colors.RED, colors.RESET });
            return;
        },
    };

    const shell_args = switch (builtin.os.tag) {
        .windows => [_][]const u8{ "powershell.exe", "-Command", command },
        .linux, .macos => [_][]const u8{ "/bin/bash", "-c", command },
        else => unreachable,
    };

    print("{s}📦 Running: {s}{s}\n", .{ colors.YELLOW, command, colors.RESET });

    var child = std.process.Child.init(&shell_args, std.heap.page_allocator);
    child.stdout_behavior = .Inherit;
    child.stderr_behavior = .Inherit;

    const result = child.spawnAndWait() catch |err| {
        print("{s}❌ Failed to run upgrade command: {any}{s}\n", .{ colors.RED, err, colors.RESET });
        return;
    };

    switch (result) {
        .Exited => |code| {
            if (code == 0) {
                print("{s}✅ Upgrade completed successfully!{s}\n", .{ colors.GREEN, colors.RESET });
            } else {
                print("{s}❌ Upgrade failed with exit code: {d}{s}\n", .{ colors.RED, code, colors.RESET });
            }
        },
        else => {
            print("{s}❌ Upgrade process was terminated unexpectedly{s}\n", .{ colors.RED, colors.RESET });
        },
    }
}
