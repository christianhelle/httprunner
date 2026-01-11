# HTTP Runner GUI - Quick Start

This guide will help you get started with the HTTP Runner GUI quickly.

## Installation

### From Release (Recommended)

Download the latest release for your platform from the [releases page](https://github.com/christianhelle/httprunner/releases):

- **Windows**: `httprunner-gui-windows-x86_64.zip`
- **macOS**: `httprunner-gui-macos-x86_64.tar.gz` or `httprunner-gui-macos-aarch64.tar.gz`
- **Linux**: `httprunner-gui-linux-x86_64.tar.gz`

Extract and run the executable.

### Build from Source

See [GUI README](README.md#building) for detailed build instructions.

## First Time Setup

1. **Launch the Application**
   ```bash
   ./httprunner-gui
   # Or on Windows: httprunner-gui.exe
   ```

2. **Open a Directory**
   - Click `File -> Open Directory...`
   - Navigate to a folder containing `.http` files
   - The file tree will populate automatically

3. **Select a File**
   - Click on any `.http` file in the left panel
   - Request details will appear in the center panel

4. **Run Requests**
   - Click "▶ Run All Requests" to execute all requests
   - OR expand a request and click "▶ Run this request"
   - Results appear in the right panel

## Using Environments

If you have an `http-client.env.json` file:

1. Select an environment from the dropdown at the top
2. Run your requests - variables will be substituted automatically

Example `http-client.env.json`:
```json
{
  "dev": {
    "baseUrl": "https://dev-api.example.com",
    "apiKey": "dev-key-123"
  },
  "prod": {
    "baseUrl": "https://api.example.com",
    "apiKey": "prod-key-456"
  }
}
```

## Example Workflow

1. **Open your project directory** with `.http` files
2. **Browse files** in the tree view (click folders to expand/collapse)
3. **Click on a file** to see its requests
4. **Expand a request** to view headers, body, etc.
5. **Select environment** (optional) if you have multiple environments
6. **Click Run** to execute
7. **View results** including status, duration, and response body

## Tips

- **Organize files in folders** - The GUI preserves your directory structure
- **Use request names** - Add `# @name myRequest` to give requests descriptive names
- **Test environments** - Switch between dev/staging/prod easily
- **Check responses** - Scroll through response bodies in the results panel
- **Multiple requests** - Run all requests in a file to test entire workflows

## Troubleshooting

### GUI won't start (Linux)

Make sure you're in a graphical environment:
```bash
echo $DISPLAY  # Should output something like :0 or :1
```

### File dialog doesn't open

This is usually a permissions issue. Try running from a terminal to see error messages:
```bash
./httprunner-gui
```

### Results not showing

Check that:
1. The request URL is valid
2. You have internet connectivity
3. The server is reachable
4. Environment variables are correctly set (if used)

## Next Steps

- Read the [full GUI README](README.md) for advanced features
- Check out [examples](../../examples/) for sample `.http` files
- Learn about [variables](../README.md#variables) and [assertions](../README.md#response-assertions)

## Feedback

Found a bug or have a suggestion? Please [open an issue](https://github.com/christianhelle/httprunner/issues)!
