const std = @import("std");
const Allocator = std.mem.Allocator;
const types = @import("types.zig");
const Variable = types.Variable;

const EnvironmentConfig = struct {
    environments: std.StringHashMap(std.StringHashMap([]const u8)),

    pub fn init(allocator: Allocator) EnvironmentConfig {
        return EnvironmentConfig{
            .environments = std.StringHashMap(std.StringHashMap([]const u8)).init(allocator),
        };
    }

    pub fn deinit(self: *EnvironmentConfig, allocator: Allocator) void {
        var env_iterator = self.environments.iterator();
        while (env_iterator.next()) |env_entry| {
            var var_iterator = env_entry.value_ptr.iterator();
            while (var_iterator.next()) |var_entry| {
                allocator.free(var_entry.key_ptr.*);
                allocator.free(var_entry.value_ptr.*);
            }
            env_entry.value_ptr.deinit();
            allocator.free(env_entry.key_ptr.*);
        }
        self.environments.deinit();
    }
};

pub fn loadEnvironmentFile(allocator: Allocator, http_file_path: []const u8, environment_name: ?[]const u8) !std.ArrayList(Variable) {
    var variables = std.ArrayList(Variable).initCapacity(allocator, 0) catch @panic("OOM");

    if (environment_name == null) {
        return variables;
    }

    const env_file_path = try findEnvironmentFile(allocator, http_file_path);
    defer if (env_file_path) |path| allocator.free(path);

    if (env_file_path == null) {
        return variables;
    }

    const env_config = parseEnvironmentFile(allocator, env_file_path.?) catch {
        // If we can't parse the environment file, just return empty variables
        // This allows the tool to continue working even if the env file has issues
        return variables;
    };
    defer {
        var config = env_config;
        config.deinit(allocator);
    }

    if (env_config.environments.get(environment_name.?)) |env_vars| {
        var iterator = env_vars.iterator();
        while (iterator.next()) |entry| {
            try variables.append(allocator, .{
                .name = try allocator.dupe(u8, entry.key_ptr.*),
                .value = try allocator.dupe(u8, entry.value_ptr.*),
            });
        }
    }

    return variables;
}

fn findEnvironmentFile(allocator: Allocator, http_file_path: []const u8) !?[]const u8 {
    const dirname = std.fs.path.dirname(http_file_path) orelse ".";

    var current_dir = try allocator.dupe(u8, dirname);
    defer allocator.free(current_dir);

    while (true) {
        const env_file_path = try std.fs.path.join(allocator, &[_][]const u8{ current_dir, "http-client.env.json" });
        defer allocator.free(env_file_path);

        if (std.fs.cwd().access(env_file_path, .{})) |_| {
            return try allocator.dupe(u8, env_file_path);
        } else |_| {
            const parent_dir = std.fs.path.dirname(current_dir);
            if (parent_dir == null or std.mem.eql(u8, parent_dir.?, current_dir)) {
                break;
            }

            const new_current_dir = try allocator.dupe(u8, parent_dir.?);
            allocator.free(current_dir);
            current_dir = new_current_dir;
        }
    }

    return null;
}

fn parseEnvironmentFile(allocator: Allocator, file_path: []const u8) !EnvironmentConfig {
    const file = try std.fs.cwd().openFile(file_path, .{});
    defer file.close();

    const file_size = try file.getEndPos();
    const content = try allocator.alloc(u8, file_size);
    defer allocator.free(content);
    _ = try file.readAll(content);

    var config = EnvironmentConfig.init(allocator);

    const parsed = std.json.parseFromSlice(std.json.Value, allocator, content, .{}) catch |err| {
        return err;
    };
    defer parsed.deinit();

    const root = parsed.value;
    if (root != .object) {
        return error.InvalidEnvironmentFile;
    }

    var env_iterator = root.object.iterator();
    while (env_iterator.next()) |env_entry| {
        const env_name = try allocator.dupe(u8, env_entry.key_ptr.*);
        var env_vars = std.StringHashMap([]const u8).init(allocator);

        if (env_entry.value_ptr.* == .object) {
            var var_iterator = env_entry.value_ptr.object.iterator();
            while (var_iterator.next()) |var_entry| {
                const var_name = try allocator.dupe(u8, var_entry.key_ptr.*);
                const var_value = switch (var_entry.value_ptr.*) {
                    .string => |s| try allocator.dupe(u8, s),
                    else => try allocator.dupe(u8, ""),
                };
                try env_vars.put(var_name, var_value);
            }
        }

        try config.environments.put(env_name, env_vars);
    }

    return config;
}
