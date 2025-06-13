# HTTP Runner

A Zig-based HTTP runner application.

## Prerequisites

- [Zig](https://ziglang.org/download/) (latest stable version recommended)

## Building

To build the project:

```bash
zig build
```

## Running

To run the application:

```bash
zig build run
```

## Testing

To run the tests:

```bash
zig build test
```

## Project Structure

```
httprunner/
├── src/
│   └── main.zig          # Main application entry point
├── build.zig             # Build configuration
├── .gitignore           # Git ignore patterns
└── README.md            # This file
```

## Development

This project follows standard Zig conventions:

- Source code is in the `src/` directory
- Build configuration is in `build.zig`
- Tests are co-located with source code using the `test` keyword
- Build artifacts are output to `zig-out/`

## License

[Add your license here]
