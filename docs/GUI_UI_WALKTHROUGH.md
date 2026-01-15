# GUI Editing Features - UI Walkthrough

This document provides a detailed walkthrough of the GUI editing interface.

## State Persistence

The GUI application automatically saves your workspace state and restores it when you restart the application. This includes:

- **Last opened directory**: The root directory you were working in
- **Selected file**: The `.http` file you had open
- **Active environment**: Your current environment selection
- **Font size**: Your preferred zoom level
- **Window size**: Your window dimensions
- **Last run results**: Your previous request execution results

### State File Location

The state is saved to a platform-specific configuration directory:
- **Windows**: `%APPDATA%\httprunner\httprunner-gui-state.json`
- **macOS**: `~/Library/Application Support/httprunner/httprunner-gui-state.json`
- **Linux**: `~/.config/httprunner/httprunner-gui-state.json`

### When State is Saved

State is automatically saved when you:
- Open a new directory
- Select a different file
- Switch environments
- Change font size (zoom in/out)
- Resize the window
- Execute requests (results are captured)
- Quit the application

You don't need to manually save anything - just close the application and your workspace will be exactly as you left it when you return.

## Main Application Window

```
┌─────────────────────────────────────────────────────────────────────────┐
│ File    Environment: [None ▼]                                           │
├───────────────┬─────────────────────────────────────────────────────────┤
│ HTTP Files    │ Request Details                                         │
│               │                                                         │
│ 📁 examples   │ 1 - GET my-test ▼                                      │
│  📄 quick.http│   Method: GET                                          │
│  📄 test.http │   URL: https://httpbin.org/get                         │
│               │   Headers:                                             │
│ 📁 tests      │     Content-Type: application/json                     │
│  📄 api.http  │                                                         │
│               │   [▶ Run] [✏ Edit] [🗑 Delete]                        │
│               │                                                         │
│               │ 2 - POST https://httpbin.org/post ▼                    │
│               │   Method: POST                                         │
│               │   URL: https://httpbin.org/post                        │
│               │   Headers:                                             │
│               │     Content-Type: application/json                     │
│               │     Authorization: Bearer token123                     │
│               │   Body:                                                │
│               │   ┌─────────────────────────────────┐                 │
│               │   │ {                               │                 │
│               │   │   "key": "value"                │                 │
│               │   │ }                               │                 │
│               │   └─────────────────────────────────┘                 │
│               │                                                         │
│               │   [▶ Run] [✏ Edit] [🗑 Delete]                        │
│               │                                                         │
│               │ [➕ Add New Request]                                   │
│               │                                                         │
│               │ [▶ Run All Requests]                                   │
│               ├─────────────────────────────────────────────────────────┤
│               │ Results                                                 │
│               │                                                         │
│               │ ✅ SUCCESS                                             │
│               │ GET https://httpbin.org/get                            │
│               │ Status: 200                                            │
│               │ Duration: 234 ms                                       │
│               │                                                         │
│               │ Response:                                              │
│               │ ┌─────────────────────────────────┐                   │
│               │ │ {                               │                   │
│               │ │   "args": {},                   │                   │
│               │ │   "headers": {...}              │                   │
│               │ │ }                               │                   │
│               │ └─────────────────────────────────┘                   │
├───────────────┴─────────────────────────────────────────────────────────┤
│ Working Directory: /home/user/projects  Selected: examples/quick.http  │
└─────────────────────────────────────────────────────────────────────────┘
```

## Edit Mode

When clicking "✏ Edit" on a request:

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Request Details                                                         │
│                                                                         │
│ Edit Request                                                            │
│ ────────────────────────────────────────────────────────────────────── │
│                                                                         │
│ Name (optional): [my-test                                           ]  │
│                                                                         │
│ Method: [GET ▼]  (Dropdown: GET, POST, PUT, DELETE, PATCH, HEAD, ...)  │
│                                                                         │
│ URL: [https://httpbin.org/get                                        ]  │
│                                                                         │
│ ────────────────────────────────────────────────────────────────────── │
│ Headers:                                                                │
│                                                                         │
│ Name: [Content-Type       ] Value: [application/json       ] [🗑]     │
│ Name: [Authorization      ] Value: [Bearer token123         ] [🗑]     │
│                                                                         │
│ [➕ Add Header]                                                        │
│                                                                         │
│ ────────────────────────────────────────────────────────────────────── │
│ Body:                                                                   │
│ ┌─────────────────────────────────────────────────────────────────────┐│
│ │{                                                                    ││
│ │  "username": "testuser",                                            ││
│ │  "password": "secret123"                                            ││
│ │}                                                                    ││
│ │                                                                     ││
│ └─────────────────────────────────────────────────────────────────────┘│
│                                                                         │
│ ────────────────────────────────────────────────────────────────────── │
│ Advanced Options ▶                                                      │
│   Timeout (ms): [5000]                                                 │
│   Connection Timeout (ms): [3000]                                      │
│   Depends On: [request-one]                                            │
│                                                                         │
│ ────────────────────────────────────────────────────────────────────── │
│                                                                         │
│ [💾 Save] [❌ Cancel]                                                  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## File Menu

```
┌─────────────────────┐
│ File                │
├─────────────────────┤
│ Open Directory...   │
│ New .http File...   │
│ ─────────────────── │
│ Quit                │
└─────────────────────┘
```

## New File Dialog Flow

1. Click "File → New .http File..."
2. File dialog appears:
   ```
   ┌────────────────────────────────────────┐
   │ Save As                                │
   ├────────────────────────────────────────┤
   │ Location: /home/user/projects          │
   │                                        │
   │ 📁 examples                            │
   │ 📁 tests                               │
   │ 📄 README.md                           │
   │                                        │
   │ File name: [new.http              ]    │
   │                                        │
   │ Files of type: [HTTP Files (*.http) ▼] │
   │                                        │
   │              [Cancel]  [Save]          │
   └────────────────────────────────────────┘
   ```
3. After saving, the new file appears in the file tree and is automatically selected
4. File contains a template request:
   ```http
   ### New Request
   GET https://httpbin.org/get
   ```

## Unsaved Changes Indicator

When there are unsaved changes:

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Request Details                                                         │
│ ...                                                                     │
│                                                                         │
│ [▶ Run All Requests] ● Unsaved changes                                 │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Key Features Illustrated

### Request List View
- Each request shows as a collapsible header
- Header displays: index, method, and name/URL
- Expanding shows full details with action buttons
- "➕ Add New Request" always visible at bottom

### Edit Mode
- Clean, form-based editing
- All fields are text inputs or dropdowns
- Headers can be added/removed dynamically
- Multi-line text area for body
- Collapsible "Advanced Options" section
- Clear Save/Cancel buttons

### File Management
- File tree shows hierarchical structure
- Icons distinguish folders (📁) from files (📄)
- New files appear immediately after creation
- Menu provides clear file operations

### Visual Feedback
- Color-coded status messages (green ✅ for success, red ❌ for errors)
- Orange dot (●) for unsaved changes indicator
- Emoji icons for intuitive button labels
- Disabled states for unavailable actions
