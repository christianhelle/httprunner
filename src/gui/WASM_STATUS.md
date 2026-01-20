# WASM Support - Current Status

## ⚠️ IMPORTANT: Work In Progress

The WebAssembly (WASM) build infrastructure has been set up, but **HTTP execution is not yet functional** in the web version.

## Current Status

### ✅ Completed

1. **Build Infrastructure**
   - WASM compilation target configured
   - Trunk build system integrated
   - Web entry point (`lib.rs`) created
   - HTML host page with loading UI

2. **Platform Adaptations**
   - State persistence using browser LocalStorage (instead of file system)
   - Conditional compilation for WASM vs native
   - WASM-specific dependencies (wasm-bindgen, web-sys, etc.)

3. **Documentation & Deployment**
   - Comprehensive build guide ([BUILD_WEB.md](BUILD_WEB.md))
   - GitHub Actions workflow for automatic deployment
   - Updated project README

4. **Dependency Fixes**
   - Downgraded `rand` from 0.9 to 0.8 for WASM compatibility
   - Added `getrandom` with `js` feature for WASM random number generation

### ❌ Blocking Issues

#### 1. **Blocking HTTP Client Not Compatible with WASM**

**Problem**: The `httprunner-lib` uses `reqwest` with the `blocking` feature, which:
- Uses synchronous I/O (not available in WASM)
- Relies on native OS threads (not available in browsers)
- Cannot compile to `wasm32-unknown-unknown` target

**Error Example**:
```
error[E0432]: unresolved imports `reqwest::blocking`
error[E0603]: module `blocking` is private
```

**Impact**: HTTP request execution will not work in the web version.

#### 2. **File System Access Limitations**

**Problem**: Browsers have limited file system access for security reasons.

**Affected Features**:
- Opening folders (requires File System Access API or manual file selection)
- File tree navigation (requires uploaded files or special APIs)
- Auto-discovery of .http files

## Path Forward

### Option A: Async Refactor (Recommended for Full Functionality)

Refactor `httprunner-lib` to support both sync and async modes:

```rust
// Native (desktop)
#[cfg(not(target_arch = "wasm32"))]
pub fn execute_request_sync(request: &Request) -> Result<Response> {
    // Use reqwest::blocking
}

// WASM (browser)
#[cfg(target_arch = "wasm32")]
pub async fn execute_request(request: &Request) -> Result<Response> {
    // Use reqwest async (works on WASM)
}
```

**Pros**:
- Full functionality in both native and web versions
- Maintains existing CLI behavior
- Future-proof architecture

**Cons**:
- Requires significant refactoring
- Changes to executor, runner, and GUI integration
- Need to update CLI to handle async (or keep dual implementations)

**Estimated Effort**: Medium (1-2 days)

### Option B: UI-Only Web Version (Quick Solution)

Deploy a web version with limited functionality:

**Available**:
- ✅ Syntax-highlighted HTTP file editor
- ✅ Request viewing and parsing
- ✅ Environment variable editing
- ✅ Copy request as cURL/code

**Not Available**:
- ❌ HTTP request execution
- ❌ Response viewing
- ❌ Folder browsing (could support file upload)

**Pros**:
- Can be deployed immediately
- Still useful as an HTTP file editor/viewer
- No library refactoring needed

**Cons**:
- Missing core functionality
- May confuse users who expect execution

**Estimated Effort**: Low (add warnings/disable run buttons)

### Option C: Progressive Web App with Service Worker

Use a service worker to handle HTTP requests from the browser:

**Challenges**:
- CORS restrictions still apply
- Cannot access localhost/private networks
- Requires proxy server for some requests

**Estimated Effort**: High (requires additional infrastructure)

## Recommended Approach

**Short-term**: Document limitations and defer web version until async refactor

**Medium-term**: Implement Option A (async support) in phases:
1. Add async execution to `httprunner-lib` alongside existing sync code
2. Update GUI to use async on WASM, sync on native
3. Keep CLI sync-based (or add async runtime)
4. Test and deploy web version

**Long-term**: Consider fully async architecture for better performance across all platforms

## Technical Details

### Dependencies Requiring Attention

1. **reqwest**: Need to use different features per platform
   ```toml
   [target.'cfg(not(target_arch = "wasm32"))'.dependencies]
   reqwest = { version = "0.12", features = ["blocking", "json", "native-tls"] }
   
   [target.'cfg(target_arch = "wasm32")'.dependencies]
   reqwest = { version = "0.12", features = ["json"], default-features = false }
   ```

2. **File system access**: `walkdir`, `rfd` have limited/no WASM support
   - May need browser file pickers
   - Could use drag-and-drop file upload

3. **State persistence**: ✅ Already handled (LocalStorage on web, files on native)

### Files Modified for WASM Support

- `src/gui/src/lib.rs` - WASM entry point
- `src/gui/src/state.rs` - Platform-specific storage
- `src/gui/index.html` - Web host page
- `src/gui/Trunk.toml` - Build configuration
- `src/gui/Cargo.toml` - WASM dependencies
- `src/lib/Cargo.toml` - Platform-specific reqwest
- `Cargo.toml` - Downgraded rand to 0.8
- `.github/workflows/web-deploy.yml` - Deployment automation

## Testing Locally

Even though execution doesn't work yet, you can test the UI build:

```bash
cd src/gui
trunk serve
```

Then open `http://127.0.0.1:8080` in your browser.

**Expected Behavior**:
- App loads and renders
- UI is responsive
- State persists in LocalStorage
- Errors when trying to execute requests

## Next Steps

1. **Decision**: Choose which option (A, B, or C) to pursue
2. **If Option A**: Create async refactor plan and implementation
3. **If Option B**: Add UI indicators showing web version limitations
4. **Testing**: Set up WASM test suite
5. **Documentation**: Update user docs once functional

## References

- [Trunk Documentation](https://trunkrs.dev/)
- [reqwest WASM Support](https://github.com/seanmonstar/reqwest/blob/master/README.md#wasm)
- [eframe Web Examples](https://github.com/emilk/egui/tree/master/examples)
- [File System Access API](https://developer.mozilla.org/en-US/docs/Web/API/File_System_Access_API)

---

**Last Updated**: 2026-01-20  
**Status**: Infrastructure Complete, Execution Blocked  
**Next Milestone**: Async HTTP Implementation
