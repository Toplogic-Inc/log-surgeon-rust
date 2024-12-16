# Schema

A schema file defines the delimiters and patterns (regular expressions) required for log-surgeon to
parse log events. Internally, log-surgeon uses these delimiters to identify tokens in the input log
stream:

- Timestamp tokens: These mark the start of a log event and must always follow the `\n` delimiter.
- Variable tokens: These must be enclosed by delimiters (exclusive).
- Static text tokens: Any text that is neither a timestamp nor a variable is treated as static text.
Technically, a static text token is also enclosed by delimiters (inclusive).


This design enables users to extract variables from unstructured log events efficiently. It also
enhances the lexer's performance by minimizing backtracking, as it primarily scans forward to locate
delimiters when splitting the text into tokens.


## Schema Configuration

The schema config, written in YAML format, allows users to define custom delimiters and
timestamp/variable patterns. The example schema config can be found [here](../examples/schema.yaml)

### Delimiters
Delimiters are defined as a string, where every character in the string is treated as an individual
delimiter.

Example:
```yaml
# Define delimiters: ' ', '\t', '\n', '\r', ':', ',', '!', ';', '%'
delimiters: " \t\r\n:,!;%"
```
**NOTE**: The newline character (`\n`) is always treated as a delimiter, even if it is not explicitly
included in the configuration.

### Timestamp Patterns
Timestamp patterns are specified as a YAML sequence. Each element in the sequence represents a
timestamp defined by a regular expression.

**Example**:
```yaml
timestamp:
  # E.g. 2015-01-31T15:50:45.392
  - '\d{4}\-\d{2}\-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}'
  # E.g. 2015-01-31T15:50:45,392
  - '\d{4}\-\d{2}\-\d{2}T\d{2}:\d{2}:\d{2},\d{3}'
  # E.g. 2015-01-31 15:50:45.392
  - '\d{4}\-\d{2}\-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}'
  # E.g. 2015-01-31 15:50:45,392
  - '\d{4}\-\d{2}\-\d{2} \d{2}:\d{2}:\d{2},\d{3}'
  # E.g. 2015-01-31 15:50:45
  - '\d{4}\-\d{2}\-\d{2} \d{2}:\d{2}:\d{2}'
```
**NOTE**: If there is ambiguity in matching timestamps, the pattern defined first in the schema file
takes precedence.

### Variable Patterns
Variable patterns are specified as a YAML mapping where each variable is uniquely identified by its
name, and its corresponding pattern is defined using a regular expression.

**Example**:
```yaml
variables:
  int: '\-{0,1}\d+'
  float: '\-{0,1}[0-9]+\.[0-9]+'
  hex: '0x(((\d|[a-f])+)|((\d|[A-F])+))'
  loglevel: '(INFO)|(DEBUG)|(WARN)|(ERROR)|(TRACE)|(FATAL)'
  thread_identifier: '\[(\w)+\]'
  path: '(/(\w|\.|\-|\*)+)+(/)*'
```
**NOTE**: If there is ambiguity in matching variables, the variable defined first in the schema file
takes precedence. For example, when matching 100, the `int` pattern will be selected over `hex`.

### Regular Expression Syntax
The following regular expression rules are supported by the schema.
```
REGEX RULE   DEFINITION
ab           Match 'a' followed by 'b'
a|b          Match a OR b
[a-z]        Match any character in the brackets (e.g., any lowercase letter)
             - special characters must be escaped, even in brackets (e.g., [\.\(\\])
a*           Match 'a' 0 or more times
a+           Match 'a' 1 or more times
a{N}         Match 'a' exactly N times
a{N,M}       Match 'a' between N and M times
(abc)        Subexpression (concatenates abc)
\d           Match any digit 0-9
\w           Match any word character ('a' to 'z', 'A' to 'Z', '0' to '9', and '_')
\s           Match any whitespace character (' ', '\r', '\t', '\v', or '\f')
.            Match any character
```

### Known Limitations
The current implementation has the following known limitations:
- The delimiters and regular expressions can only contain ASCII characters.
- If a variable contains any delimiters, it might trigger undefined parsing results if there are
partially-matched patterns in the input log stream (tracked in [this][gh-issue] GitHub issue.)

[gh-issue]: https://github.com/Toplogic-Inc/log-surgeon-rust/issues/14