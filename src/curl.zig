const std = @import("std");
const Allocator = std.mem.Allocator;

// libcurl C bindings
pub const CURL = opaque {};
pub const CURLcode = c_int;
pub const CURLoption = c_int;

pub const CURLOPT_URL: CURLoption = 10002;
pub const CURLOPT_WRITEFUNCTION: CURLoption = 20011;
pub const CURLOPT_WRITEDATA: CURLoption = 10001;
pub const CURLOPT_HEADERFUNCTION: CURLoption = 20079;
pub const CURLOPT_HEADERDATA: CURLoption = 10029;
pub const CURLOPT_CUSTOMREQUEST: CURLoption = 10036;
pub const CURLOPT_POSTFIELDS: CURLoption = 10015;
pub const CURLOPT_POSTFIELDSIZE: CURLoption = 60;
pub const CURLOPT_HTTPHEADER: CURLoption = 10023;
pub const CURLOPT_SSL_VERIFYPEER: CURLoption = 64;
pub const CURLOPT_SSL_VERIFYHOST: CURLoption = 81;

pub const CURLE_OK: CURLcode = 0;

extern "c" fn curl_global_init(flags: c_long) CURLcode;
extern "c" fn curl_global_cleanup() void;
extern "c" fn curl_easy_init() ?*CURL;
extern "c" fn curl_easy_cleanup(curl: *CURL) void;
extern "c" fn curl_easy_setopt(curl: *CURL, option: CURLoption, ...) CURLcode;
extern "c" fn curl_easy_perform(curl: *CURL) CURLcode;
extern "c" fn curl_easy_getinfo(curl: *CURL, info: c_int, ...) CURLcode;
extern "c" fn curl_slist_append(list: ?*curl_slist, string: [*:0]const u8) ?*curl_slist;
extern "c" fn curl_slist_free_all(list: ?*curl_slist) void;

pub const curl_slist = extern struct {
    data: [*:0]u8,
    next: ?*curl_slist,
};

pub const CURLINFO_RESPONSE_CODE: c_int = 0x200000 + 2;

const WriteCallbackData = struct {
    buffer: *std.ArrayList(u8),
    allocator: Allocator,
};

const HeaderCallbackData = struct {
    headers: *std.ArrayList(Header),
    allocator: Allocator,
};

pub const Header = struct {
    name: []const u8,
    value: []const u8,
};

pub fn init() !void {
    const result = curl_global_init(3); // CURL_GLOBAL_ALL = 3
    if (result != CURLE_OK) {
        return error.CurlInitFailed;
    }
}

pub fn deinit() void {
    curl_global_cleanup();
}

fn writeCallback(contents: [*]u8, size: usize, nmemb: usize, userdata: *anyopaque) callconv(.C) usize {
    const data: *WriteCallbackData = @ptrCast(@alignCast(userdata));
    const realsize = size * nmemb;
    
    data.buffer.appendSlice(contents[0..realsize]) catch return 0;
    
    return realsize;
}

fn headerCallback(contents: [*]u8, size: usize, nmemb: usize, userdata: *anyopaque) callconv(.C) usize {
    const data: *HeaderCallbackData = @ptrCast(@alignCast(userdata));
    const realsize = size * nmemb;
    const header_line = contents[0..realsize];
    
    // Parse header line
    if (std.mem.indexOf(u8, header_line, ":")) |colon_pos| {
        const name = std.mem.trim(u8, header_line[0..colon_pos], " \t\r\n");
        const value = std.mem.trim(u8, header_line[colon_pos + 1..], " \t\r\n");
        
        if (name.len > 0) {
            const name_copy = data.allocator.dupe(u8, name) catch return 0;
            const value_copy = data.allocator.dupe(u8, value) catch {
                data.allocator.free(name_copy);
                return 0;
            };
            
            data.headers.append(Header{
                .name = name_copy,
                .value = value_copy,
            }) catch {
                data.allocator.free(name_copy);
                data.allocator.free(value_copy);
                return 0;
            };
        }
    }
    
    return realsize;
}

pub const CurlRequest = struct {
    url: []const u8,
    method: []const u8,
    headers: []const Header,
    body: ?[]const u8,
    insecure: bool,
};

pub const CurlResponse = struct {
    status_code: u16,
    headers: []Header,
    body: []u8,
    allocator: Allocator,
    
    pub fn deinit(self: *CurlResponse) void {
        for (self.headers) |header| {
            self.allocator.free(header.name);
            self.allocator.free(header.value);
        }
        self.allocator.free(self.headers);
        self.allocator.free(self.body);
    }
};

pub fn perform(allocator: Allocator, request: CurlRequest) !CurlResponse {
    const curl = curl_easy_init() orelse return error.CurlInitFailed;
    defer curl_easy_cleanup(curl);
    
    // Convert URL to null-terminated string
    const url_z = try allocator.dupeZ(u8, request.url);
    defer allocator.free(url_z);
    
    _ = curl_easy_setopt(curl, CURLOPT_URL, url_z.ptr);
    
    // Set insecure option if requested
    if (request.insecure) {
        _ = curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, @as(c_long, 0));
        _ = curl_easy_setopt(curl, CURLOPT_SSL_VERIFYHOST, @as(c_long, 0));
    }
    
    // Set HTTP method
    if (!std.mem.eql(u8, request.method, "GET")) {
        const method_z = try allocator.dupeZ(u8, request.method);
        defer allocator.free(method_z);
        _ = curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, method_z.ptr);
    }
    
    // Set headers
    var slist: ?*curl_slist = null;
    defer curl_slist_free_all(slist);
    
    for (request.headers) |header| {
        const header_str = try std.fmt.allocPrintZ(allocator, "{s}: {s}", .{header.name, header.value});
        defer allocator.free(header_str);
        slist = curl_slist_append(slist, header_str.ptr);
    }
    
    if (slist != null) {
        _ = curl_easy_setopt(curl, CURLOPT_HTTPHEADER, slist);
    }
    
    // Set body if present
    if (request.body) |body| {
        _ = curl_easy_setopt(curl, CURLOPT_POSTFIELDS, body.ptr);
        _ = curl_easy_setopt(curl, CURLOPT_POSTFIELDSIZE, @as(c_long, @intCast(body.len)));
    }
    
    // Setup response body callback
    var body_buffer = std.ArrayList(u8).init(allocator);
    var write_data = WriteCallbackData{
        .buffer = &body_buffer,
        .allocator = allocator,
    };
    
    _ = curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, writeCallback);
    _ = curl_easy_setopt(curl, CURLOPT_WRITEDATA, &write_data);
    
    // Setup response headers callback
    var headers_list = std.ArrayList(Header).init(allocator);
    var header_data = HeaderCallbackData{
        .headers = &headers_list,
        .allocator = allocator,
    };
    
    _ = curl_easy_setopt(curl, CURLOPT_HEADERFUNCTION, headerCallback);
    _ = curl_easy_setopt(curl, CURLOPT_HEADERDATA, &header_data);
    
    // Perform request
    const res = curl_easy_perform(curl);
    if (res != CURLE_OK) {
        body_buffer.deinit();
        for (headers_list.items) |header| {
            allocator.free(header.name);
            allocator.free(header.value);
        }
        headers_list.deinit();
        return error.CurlPerformFailed;
    }
    
    // Get response code
    var response_code: c_long = 0;
    _ = curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &response_code);
    
    const headers_owned = try headers_list.toOwnedSlice();
    const body_owned = try body_buffer.toOwnedSlice();
    
    return CurlResponse{
        .status_code = @intCast(response_code),
        .headers = headers_owned,
        .body = body_owned,
        .allocator = allocator,
    };
}
