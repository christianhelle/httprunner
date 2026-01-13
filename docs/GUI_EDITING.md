# GUI Editing Features

The HTTP File Runner GUI now supports full editing capabilities for .http files. This document describes the new features.

## Features

### 1. Editing Existing Requests

When viewing a .http file in the GUI, you can now:

- **Edit Request**: Click the "✏ Edit" button on any request to enter edit mode
- **Modify Fields**:
  - Request Name (optional identifier)
  - HTTP Method (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
  - URL
  - Headers (add/remove/edit name-value pairs)
  - Request Body
  - Advanced options: Timeout, Connection Timeout, Dependencies

- **Save Changes**: Click "💾 Save" to persist changes to the file
- **Cancel Editing**: Click "❌ Cancel" to discard changes

### 2. Adding New Requests

- Click the "➕ Add New Request" button at the bottom of the request list
- Fill in the request details in the editor
- Click "💾 Save" to add the request to the file

### 3. Deleting Requests

- Click the "🗑 Delete" button on any request to remove it from the file
- The file is automatically saved after deletion

### 4. Creating New .http Files

- Go to **File → New .http File...** in the menu bar
- Choose a location and filename
- A new file will be created with a template GET request
- The file tree automatically refreshes to show the new file

### 5. File Management

- The GUI shows "● Unsaved changes" indicator when there are pending modifications
- Files are automatically saved when you:
  - Save an edited request
  - Delete a request
  - Create a new request
- The file tree refreshes after creating new files

## Usage Example

1. **Open Directory**: File → Open Directory... and select a folder containing .http files
2. **Select File**: Click on a .http file in the left panel
3. **Edit Request**: 
   - Click "✏ Edit" on any request
   - Modify the URL, headers, or body
   - Click "💾 Save"
4. **Add New Request**:
   - Click "➕ Add New Request"
   - Set method to POST
   - Enter URL: https://api.example.com/users
   - Add header: `Content-Type: application/json`
   - Add body: `{"name": "John Doe"}`
   - Click "💾 Save"
5. **Run Requests**: Use the "▶ Run" button or "▶ Run All Requests" to execute

## File Format

The GUI serializes requests back to the standard .http format:

```http
###
# @name request-name
# @timeout 5000ms
# @dependsOn other-request
POST https://api.example.com/endpoint
Content-Type: application/json
Authorization: Bearer token

{
  "key": "value"
}
```

All standard directives are preserved:
- `@name` - Request identifier
- `@timeout` - Request timeout in milliseconds
- `@connection-timeout` - Connection timeout in milliseconds
- `@dependsOn` - Request dependency
- `@if` - Conditional execution (preserved but not editable in GUI yet)
- `@assert` - Assertions (preserved but not editable in GUI yet)

## Keyboard Shortcuts

- **Ctrl + Plus**: Zoom in (increase font size)
- **Ctrl + Minus**: Zoom out (decrease font size)
- **Ctrl + 0**: Reset font size to default

## Technical Details

The editing functionality is implemented using:
- `RequestEditor`: Manages the editing state and file operations
- `Serializer`: Converts `HttpRequest` objects back to .http file format
- `Parser`: Parses .http files into `HttpRequest` objects

The round-trip (parse → edit → serialize → parse) is fully tested and maintains all request properties.
