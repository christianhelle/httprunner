# JSONPath Array Indexing Fix

## Problem
Request variable substitution with JSONPath array indexing was not working. When using syntax like:
```
{{get_versions.response.body.$.data[0].version}}
```
The variable was not being substituted, and the URL would contain the literal variable reference instead of the extracted value.

## Root Cause
The `extract_json_property` function in `src/request_variables.rs` was splitting the JSONPath by `.` and processing each part sequentially, but it didn't have logic to handle array indexing syntax like `data[0]`.

When parsing `$.data[0].version`, it would try to find a property literally named `data[0]` instead of:
1. Finding the `data` property (which is an array)
2. Extracting element at index `0`
3. Then extracting the `version` property from that element

## Solution
Added array indexing support to the JSONPath parser:

### Changes Made

1. **Modified `extract_json_property` function** to detect array indexing syntax:
   - Check if a path part contains `[` 
   - Split into property name and index part (e.g., `data[0]` → `data` + `[0]`)
   - First extract the property, then extract the array element

2. **Added `parse_array_index` function**:
   - Parses array index syntax like `[0]`, `[1]`, etc.
   - Returns the numeric index

3. **Added `extract_array_element` function**:
   - Takes a JSON array string and an index
   - Properly handles nested objects and arrays within the array
   - Handles string escaping and nested brackets/braces
   - Returns the element at the specified index

4. **Enhanced `extract_simple_json_property` function**:
   - Added support for array values (starting with `[`)
   - Properly tracks bracket depth while handling strings
   - Handles escape sequences in strings

## Examples

### Basic Array Access
```http
# @name get_data
POST https://api.example.com/data
Content-Type: application/json

{
  "items": [
    {"id": 1, "name": "first"},
    {"id": 2, "name": "second"}
  ]
}

###

# Access first item's name
GET https://api.example.com/item/{{get_data.response.body.$.json.items[0].name}}
# Result: https://api.example.com/item/first
```

### Nested Array Access
```http
# @name get_versions
POST https://api.example.com/versions
Content-Type: application/json

{
  "data": {
    "versions": [
      {"version": "2.3.0", "url": "http://example.com/v2.3.0"}
    ]
  }
}

###

# Use nested array data
GET https://api.example.com/versions/{{get_versions.response.body.$.json.data.versions[0].version}}
# Result: https://api.example.com/versions/2.3.0
```

### Multiple Array Indices
```http
GET https://api.example.com/compare?v1={{get_data.response.body.$.json.items[0].id}}&v2={{get_data.response.body.$.json.items[1].id}}
# Result: https://api.example.com/compare?v1=1&v2=2
```

## Testing
Created test files to verify the fix:
- `examples/test-array-jsonpath.http` - Basic array indexing
- `examples/simple-array-test.http` - User's specific scenario
- `examples/comprehensive-array-test.http` - Various edge cases

All tests pass successfully with correct variable substitution.

## Limitations
- Currently only supports numeric array indices (e.g., `[0]`, `[1]`)
- Does not support array filter expressions or wildcards
- Direct indexing of primitive arrays (e.g., `["a","b","c"][0]`) requires the array to be a property value first

## Future Enhancements
Could add support for:
- Negative indices (e.g., `[-1]` for last element)
- Array slicing (e.g., `[0:2]`)
- Filter expressions (e.g., `[?(@.status=='active')]`)
