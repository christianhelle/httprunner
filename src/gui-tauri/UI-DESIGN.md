# Tauri GUI User Interface

## Overview

The Tauri GUI provides a modern, web-based interface for HTTP File Runner with a clean, intuitive layout similar to popular REST clients like Postman or Insomnia.

## Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│ HTTP File Runner                    📁 Open Directory   Environment: ▼  │ Header
├──────────────┬──────────────────────────────────────────────────────────┤
│              │ Request Details                           💾 Save        │
│ HTTP Files   ├────────────────────────────────────┬─────────────────────┤
│              │                                    │                     │
│ ├─ api/      │ 1. GET /users                     │ ### Get All Users   │
│    └─ users  │    GET https://api.example.com    │ GET https://api...  │
│       .http  │    [▶ Run]                        │                     │
│              │                                    │ Accept: applicat... │
│ ├─ test/     │ 2. POST /users                    │                     │
│    └─ basic  │    POST https://api.example.com   │ {                   │
│       .http  │    [▶ Run]                        │   "name": "John",   │
│              │                                    │   "email": "..."    │
│ (selected)   │ 3. GET /users/:id                 │ }                   │
│ example.http │    GET https://api.example.com    │                     │
│              │    [▶ Run]                        │                     │
│              ├────────────────────────────────────┴─────────────────────┤
│              │ [▶ Run All Requests]                Status: Ready        │
├──────────────┼──────────────────────────────────────────────────────────┤
│              │ Results                                                  │
│              ├──────────────────────────────────────────────────────────┤
│              │ ✅ GET /users                             200 (234ms)    │
│              │ https://api.example.com/users                            │
│              │ ┌────────────────────────────────────────────────────┐   │
│              │ │ {                                                  │   │
│              │ │   "users": [                                       │   │
│              │ │     { "id": 1, "name": "John" },                  │   │
│              │ │     { "id": 2, "name": "Jane" }                   │   │
│              │ │   ]                                                │   │
│              │ │ }                                                  │   │
│              │ └────────────────────────────────────────────────────┘   │
├──────────────┴──────────────────────────────────────────────────────────┤
│ Working Directory: /home/user/projects/api                              │ Footer
│ Selected: /home/user/projects/api/example.http                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Component Description

### Header (Top Bar)
- **App Title**: "HTTP File Runner" with emoji icon 🌐
- **Open Directory Button**: Opens a folder picker to select working directory
- **Environment Selector**: Dropdown to select from available environments (None, Dev, Staging, Prod, etc.)
- Background: Dark blue (#2c3e50)
- Text: White

### Sidebar (Left Panel - 300px wide)
- **Title**: "HTTP Files"
- **File List**: 
  - Displays all `.http` files found in the directory tree
  - Files are shown with indentation for directory structure
  - Selected file is highlighted in blue
  - Clickable items to select a file
  - Auto-scrollable if list is long

### Request Details Panel (Top Center - Resizable)
- **Left Section** (300px):
  - **Requests List**: Shows all parsed requests from the selected file
  - Each request displays:
    - Index number
    - Request name (if provided)
    - HTTP Method (colored: GET=blue, POST=green, PUT=orange, DELETE=red)
    - URL
    - Individual "▶ Run" button
  - Scrollable list

- **Right Section** (Flexible):
  - **File Editor**: Text area showing the raw `.http` file content
  - Editable with monospace font
  - Shows unsaved changes indicator
  - Save button (enabled only when modified)

### Action Bar
- **Run All Requests Button**: Green button to execute all requests
- **Status Message**: Shows current operation status
- Disabled when file has unsaved changes

### Results Panel (Bottom Center - Resizable)
- **Title**: "Results"
- **Results List**: Shows execution results for each request
  - Success results (green background):
    - Method and URL
    - Status code and duration
    - Response body (formatted JSON if applicable)
  - Failure results (red background):
    - Method and URL
    - Error message
  - Each result is in a card with rounded corners
  - Scrollable list

### Footer (Bottom Bar)
- **Working Directory**: Shows current working directory path
- **Selected File**: Shows path of currently selected file
- Background: Dark blue (#2c3e50)
- Text: White
- Font: Smaller (12px)

## Color Scheme

### Primary Colors
- **Background**: #f5f5f5 (light gray)
- **Panel Background**: #ffffff (white)
- **Header/Footer**: #2c3e50 (dark blue)
- **Primary Action**: #3498db (blue)
- **Success**: #27ae60 (green)
- **Error**: #e74c3c (red)
- **Warning**: #f39c12 (orange)

### Text Colors
- **Primary**: #333333 (dark gray)
- **Secondary**: #666666 (medium gray)
- **Muted**: #999999 (light gray)
- **Inverse**: #ffffff (white, for dark backgrounds)

### HTTP Method Colors
- **GET**: #3498db (blue)
- **POST**: #27ae60 (green)
- **PUT**: #f39c12 (orange)
- **PATCH**: #9b59b6 (purple)
- **DELETE**: #e74c3c (red)

## Interactions

### File Selection
1. User clicks on a file in the sidebar
2. File becomes highlighted
3. File content loads in the editor
4. Requests are parsed and displayed in the requests list
5. Environments are loaded for the file

### Running Single Request
1. User clicks "▶ Run" on a request
2. Status shows "Running request..."
3. Request executes
4. Result appears at the top of the results panel
5. Status shows "Request completed"

### Running All Requests
1. User clicks "▶ Run All Requests"
2. Status shows "Running all requests..."
3. All requests execute sequentially
4. Results appear in the results panel as they complete
5. Status shows "Completed X requests"

### Editing File
1. User modifies text in the editor
2. Save button becomes enabled
3. Run All button becomes disabled
4. User clicks Save
5. File is written to disk
6. Requests are re-parsed
7. Save button disabled, Run All enabled

### Environment Selection
1. User selects an environment from dropdown
2. Selection is saved
3. Environment variables will be used in next request execution

## Responsive Design

- Minimum window size: 800x600
- Default window size: 1200x800
- Resizable panels:
  - Sidebar can be resized horizontally
  - Request details/results panels can be resized vertically
- Scrollable content areas prevent overflow

## Accessibility Features

- Clear visual hierarchy
- Sufficient color contrast
- Keyboard navigation support (via web standards)
- Focus indicators
- Semantic HTML structure

## Future Enhancements

The UI is designed to accommodate future features:
- Syntax highlighting in the editor
- Multiple tabs for different files
- Request history sidebar
- Dark mode toggle
- Export functionality
- Settings panel
- GraphQL support
- WebSocket connections visualization
