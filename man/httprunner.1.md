% httprunner(1) | General Commands Manual
% Christian Helle
% July 2024

# NAME

**httprunner** - A simple command-line tool that parses .http files and executes HTTP requests.

# SYNOPSIS

**httprunner** <http-file> [http-file2] [...] [--verbose] [--log [filename]] [--env <environment>]  
**httprunner** [--verbose] [--log [filename]] [--env <environment>] --discover  
**httprunner** --version | -v  
**httprunner** --upgrade  
**httprunner** --help | -h

# DESCRIPTION

**httprunner** is a command-line tool written in Zig that parses `.http` files and executes HTTP requests, providing colored output with emojis to indicate success or failure.

# OPTIONS

**<http-file>**
: One or more .http files to process.

**--discover**
: Recursively discover and process all .http files from the current directory.

**--verbose**
: Show detailed HTTP request and response information.

**--log** [filename]
: Log output to a file (defaults to 'log' if no filename is specified).

**--env** <environment>
: Specify environment name to load variables from http-client.env.json.

**--version**, **-v**
: Show version information.

**--upgrade**
: Update httprunner to the latest version.

**--help**, **-h**
: Show this help message.

# EXAMPLES

**Run a single .http file**
: `httprunner examples/simple.http`

**Run a single .http file with verbose output**
: `httprunner examples/simple.http --verbose`

**Run multiple .http files**
: `httprunner examples/simple.http examples/quick.http`

**Discover and run all .http files recursively**
: `httprunner --discover`

**Use a specific environment from http-client.env.json**
: `httprunner myfile.http --env dev`

# FILES

**http-client.env.json**
: To give variables different values in different environments, create a file named `http-client.env.json`. This file should be located in the same directory as the `.http` file or in one of its parent directories.

# AUTHOR

Christian Helle

# COPYRIGHT

This project is open source and available under the MIT License.
