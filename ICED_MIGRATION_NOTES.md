# Iced Migration Notes

## Overview

The GUI has been successfully ported from egui/eframe to Iced framework v0.14.0.

## Changes Made

### Dependencies
- Removed: `egui`, `eframe`
- Added: `iced` (v0.14.0), `image` (v0.25)

### Architecture
The application now follows Iced's Elm-inspired architecture:

1. **State** (`HttpRunnerApp`): Holds all application state
2. **Messages** (`Message` enum): Defines all possible user interactions
3. **Update** (`update()` method): Pure function that modifies state based on messages, returns `Task<Message>` for side effects
4. **View** (`view()` method): Pure function that renders UI based on current state

### Key Differences from egui

| Feature | egui/eframe | Iced |
|---------|-------------|------|
| UI Pattern | Immediate mode | Retained/Declarative |
| State | Mutable within update | Immutable snapshots |
| Events | Callback-based | Message-based |
| Layout | Flexible panels | Widget composition |
| Async | Manual threading | Task/Command system |
| Styling | Style structs | Theme system |

### File-by-File Changes

#### `main.rs`
- Uses `iced::application()` builder instead of `eframe::run_native()`
- Simplified to just configure and run the application
- Icon loading temporarily removed (needs Iced-specific implementation)

#### `app.rs`
- Added `Message` enum for all UI events
- `update()` returns `Task<Message>` instead of being void
- `view()` returns `Element<'_, Message>` instead of mutating UI
- `subscription()` handles keyboard and window events
- All UI construction moved from imperative to declarative style

#### `file_tree.rs`
- Converted from egui widgets to Iced widgets
- Added `Message` enum for file selection
- `update()` method returns selected file
- `view()` builds widget tree declaratively

#### `request_view.rs`
- Simplified editor UI (full editor implementation deferred)
- Added `Message` enum for request actions
- Fixed lifetime annotations in `Element<'_, Message>` return types
- String cloning necessary to avoid borrowing local variables

#### `results_view.rs`
- Converted display from egui to Iced widgets
- Fixed API call to `runner::execute_http_request()`
- View returns proper `Element` with explicit lifetime

### Testing

The application compiles successfully. To test locally with a display:

```bash
cargo run --package httprunner-gui
```

**Note**: In CI/headless environments, the app will fail with:
```
neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set.
```

This is expected and does not indicate a problem with the port.

### Known Limitations

1. **Icon Loading**: The application icon is not currently loaded. Iced 0.14 requires a different approach to icon handling.

2. **Request Editor**: The full request editing UI is simplified in this port. Users can:
   - View all requests in a file
   - Run individual requests
   - Delete requests
   - The inline editor for modifying requests is a placeholder

3. **Font Size Control**: The font size adjustment feature from egui needs to be reimplemented using Iced's theming system.

4. **Environment Selector**: The environment dropdown is simplified to display only - switching works via keyboard shortcut (Ctrl+E).

### Future Enhancements

1. Implement full request editor with Iced's text input widgets
2. Add proper icon support using `iced::window::icon::from_rgba`
3. Implement custom theme with configurable font sizes
4. Add environment dropdown using Iced's `pick_list` widget
5. Improve layout with proper resizable panes
6. Add syntax highlighting for request bodies using custom widgets

### Benefits of Iced

1. **Type Safety**: Stronger compile-time guarantees with message-based architecture
2. **Predictability**: Pure functions make UI behavior more predictable
3. **Testing**: Easier to test UI logic without rendering
4. **Performance**: Retained mode can be more efficient for complex UIs
5. **Cross-platform**: Better Wayland support out of the box
6. **Modern**: More actively developed with cleaner API

## Verification

Build output:
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.26s
```

Warnings: Only 1 dead code warning for unused editor method (expected).

Errors: None ✅
