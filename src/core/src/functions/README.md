# Functions Module

This module handles the substitution of dynamic function calls in HTTP request files, including random data generation and encoding utilities.

## Structure

- `mod.rs` - Module entry point and public API
- `substitution.rs` - Function substitution trait and orchestration
- `generator_functions.rs` - Built-in generator function implementations
- `date_functions.rs` - Built-in date function implementations
- `transform_functions.rs` - Built-in transformation function implementations
- `tests.rs` - Test suite

## Usage

```rust
use crate::functions::substitute_functions;

let url = substitute_functions("https://api.example.com/users/{{guid()}}")?;
let body = substitute_functions(r#"{"id": "{{guid()}}", "name": "{{string()}}"}"#)?;
```

## Supported Functions

### GUID Generation

Generates a random UUID v4 in simple format (no hyphens):

```
{{guid()}}
```

Example output: `a1b2c3d4e5f67890a1b2c3d4e5f67890`

### Random String

Generates a random 20-character alphanumeric string:

```
{{string()}}
```

Example output: `aB3dE5fG7hI9jK1lM3nO`

### Random Number

Generates a random integer between 0 and 100:

```
{{number()}}
```

Example output: `42`

### Base64 Encoding

Encodes a string value to Base64:

```
{{base64_encode('value to encode')}}
```

Example: `{{base64_encode('username:password')}}` → `dXNlcm5hbWU6cGFzc3dvcmQ=`

### Uppercase Transformation

Converts a string to uppercase:

```text
{{upper('text to convert')}}
```

Example: `{{upper('hello, world')}}` → `HELLO, WORLD`

### Lowercase Transformation

Converts a string to lowercase:

```text
{{lower('TEXT TO CONVERT')}}
```

Example: `{{lower('HELLO, WORLD')}}` → `hello, world`

### Name Generation

Generates a random full name (first name + last name):

```
{{name()}}
```

Example output: `John Smith`

### First Name Generation

Generates a random first name:

```
{{first_name()}}
```

Example output: `Sarah`

### Last Name Generation

Generates a random last name:

```
{{last_name()}}
```

Example output: `Johnson`

### Address Generation

Generates a random full mailing address (street, city, postal code, country):

```
{{address()}}
```

Example output: `123 Main Street, Springfield, 12345, USA`

### Email Generation

Generates a random email address:

```
{{email()}}
```

Example output: `john.smith@example.com`

### Job Title Generation

Generates a random job title:

```
{{job_title()}}
```

Example output: `Senior Software Engineer`

### Lorem Ipsum Text Generation

Generates Lorem Ipsum placeholder text with a specified number of words:

```text
{{lorem_ipsum(50)}}
```

Example output: `lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magna aliqua enim ad minim veniam quis nostrud exercitation ullamco laboris nisi aliquip ex ea commodo consequat duis aute irure in reprehenderit voluptate velit esse cillum fugiat nulla pariatur`

The function accepts a parameter specifying the number of words to generate. If the requested number exceeds the available word list, words will be repeated cyclically.

## Implementation Details

Functions are case-insensitive and processed through the `FunctionSubstitutor` trait, which provides:

- Pattern matching via regex
- Value generation or transformation
- String replacement logic

All functions are applied sequentially to the input text, allowing for multiple function calls in a single string.
