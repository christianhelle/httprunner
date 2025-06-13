const std = @import("std");
const print = std.debug.print;
const Allocator = std.mem.Allocator;

// ANSI color codes
const RED = "\x1b[31m";
const GREEN = "\x1b[32m";
const YELLOW = "\x1b[33m";
const BLUE = "\x1b[34m";
const RESET = "\x1b[0m";

const HttpRequest = struct {
    method: []const u8,
    url: []const u8,
    headers: std.ArrayList(Header),
    body: ?[]const u8,

    const Header = struct {
        name: []const u8,
        value: []const u8,
    };    fn deinit(self: *HttpRequest, allocator: Allocator) void {
        allocator.free(self.method);
        allocator.free(self.url);
        
        for (self.headers.items) |header| {
            allocator.free(header.name);
            allocator.free(header.value);
        }
        self.headers.deinit();
        
        if (self.body) |body| {
            allocator.free(body);
        }
    }
};

const HttpResult = struct {
    status_code: u16,
    success: bool,
    error_message: ?[]const u8,
};

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 2) {
        print("{s}❌ Usage: {s} <http-file>{s}\n", .{ RED, args[0], RESET });
        return;
    }

    const http_file = args[1];    print("{s}🚀 HTTP Runner - Processing file: {s}{s}\n", .{ BLUE, http_file, RESET });
    print("{s}\n", .{"=" ** 50});

    const requests = parseHttpFile(allocator, http_file) catch |err| {
        switch (err) {
            error.FileNotFound => {
                print("{s}❌ Error: File '{s}' not found{s}\n", .{ RED, http_file, RESET });
                return;
            },
            else => {
                print("{s}❌ Error parsing file: {}{s}\n", .{ RED, err, RESET });
                return;
            },
        }
    };    defer {
        for (requests.items) |*req| {
            req.deinit(allocator);
        }
        requests.deinit();
    }

    if (requests.items.len == 0) {
        print("{s}⚠️  No HTTP requests found in file{s}\n", .{ YELLOW, RESET });
        return;
    }

    print("Found {} HTTP request(s)\n\n", .{requests.items.len});
}

fn parseHttpFile(allocator: Allocator, file_path: []const u8) !std.ArrayList(HttpRequest) {
    const file = std.fs.cwd().openFile(file_path, .{}) catch |err| {
        return err;
    };
    defer file.close();

    const file_size = try file.getEndPos();
    const content = try allocator.alloc(u8, file_size);
    defer allocator.free(content);
    _ = try file.readAll(content);    var requests = std.ArrayList(HttpRequest).init(allocator);
    var lines = std.mem.splitSequence(u8, content, "\n");
    
    var current_request: ?HttpRequest = null;
    var in_body = false;
    var body_lines = std.ArrayList([]const u8).init(allocator);
    defer body_lines.deinit();

    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t\r\n");
        
        // Skip empty lines and comments
        if (trimmed.len == 0 or std.mem.startsWith(u8, trimmed, "#")) {
            continue;
        }

        // Check if this is a new HTTP request line
        if (std.mem.indexOf(u8, trimmed, "HTTP/") != null or 
            std.mem.startsWith(u8, trimmed, "GET ") or
            std.mem.startsWith(u8, trimmed, "POST ") or
            std.mem.startsWith(u8, trimmed, "PUT ") or
            std.mem.startsWith(u8, trimmed, "DELETE ") or
            std.mem.startsWith(u8, trimmed, "PATCH ")) {
            
            // Save previous request if exists
            if (current_request) |*req| {
                if (body_lines.items.len > 0) {
                    const body = try std.mem.join(allocator, "\n", body_lines.items);
                    req.body = body;
                }
                try requests.append(req.*);
                body_lines.clearRetainingCapacity();
            }
              // Parse new request
            var parts = std.mem.splitSequence(u8, trimmed, " ");
            const method = parts.next() orelse return error.InvalidRequest;
            const url = parts.next() orelse return error.InvalidRequest;
            
            current_request = HttpRequest{
                .method = try allocator.dupe(u8, method),
                .url = try allocator.dupe(u8, url),
                .headers = std.ArrayList(HttpRequest.Header).init(allocator),
                .body = null,
            };
            in_body = false;
        } else if (std.mem.indexOf(u8, trimmed, ":") != null and !in_body) {            // Parse header
            if (current_request) |*req| {
                var header_parts = std.mem.splitSequence(u8, trimmed, ":");
                const name = std.mem.trim(u8, header_parts.next() orelse "", " \t");
                const value = std.mem.trim(u8, header_parts.rest(), " \t");
                
                try req.headers.append(.{
                    .name = try allocator.dupe(u8, name),
                    .value = try allocator.dupe(u8, value),
                });
            }
        } else {
            // Body content
            in_body = true;
            try body_lines.append(try allocator.dupe(u8, trimmed));
        }
    }

    // Save last request
    if (current_request) |*req| {
        if (body_lines.items.len > 0) {
            const body = try std.mem.join(allocator, "\n", body_lines.items);
            req.body = body;
        }
        try requests.append(req.*);
    }

    return requests;
}

fn executeHttpRequest(allocator: Allocator, request: HttpRequest) !HttpResult {
    // Parse URL to extract scheme, host, port, and path
    const uri = std.Uri.parse(request.url) catch {
        return HttpResult{
            .status_code = 0,
            .success = false,
            .error_message = "Invalid URL",
        };
    };

    var client = http.Client{ .allocator = allocator };
    defer client.deinit();

    // Convert method string to enum
    const method = std.meta.stringToEnum(http.Method, request.method) orelse {
        return HttpResult{
            .status_code = 0,
            .success = false,
            .error_message = "Invalid HTTP method",
        };
    };    // Prepare headers
    var header_buffer: [8192]u8 = undefined;
    var req = client.open(method, uri, .{
        .server_header_buffer = &header_buffer,
    }) catch |err| {
        return HttpResult{
            .status_code = 0,
            .success = false,
            .error_message = switch (err) {
                error.UnknownHostName => "Unknown host",
                error.ConnectionRefused => "Connection refused",
                error.NetworkUnreachable => "Network unreachable",
                else => "Connection error",
            },
        };
    };
    defer req.deinit();

    // TODO: Add custom headers support for Zig 0.14
    // For now, we'll skip custom headers to get the basic functionality working

    try req.send();

    // Send body if present
    if (request.body) |body| {
        try req.writeAll(body);
    }

    try req.finish();
    try req.wait();

    const status_code = @intFromEnum(req.response.status);
    const success = status_code >= 200 and status_code < 300;

    return HttpResult{
        .status_code = status_code,
        .success = success,
        .error_message = null,
    };
}

test "simple test" {
    var list = std.ArrayList(i32).init(std.testing.allocator);
    defer list.deinit(); // try commenting this out and see if zig detects the memory leak!
    try list.append(42);
    try std.testing.expectEqual(@as(i32, 42), list.pop());
}
