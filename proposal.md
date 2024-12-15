# log-surgeon: A performant log parsing library
Project Link: [Homepage][home-page]

## Team Members
- Student 1: Siwei (Louis) He, 1004220960
- Student 2: Zhihao Lin, 1005071299

## Introduction

`log-surgeon` is a library for high-performance parsing of unstructured text
logs implemented using Rust.


## Motivation
Today's large technology companies generate logs the magnitude of petabytes per day as a critical
source for runtime failure diagnostics and data analytics. In a real-world production environment,
logs can be split into two categories: unstructured logs and structured logs, where unstructured logs
usually consist of a timestamp and a raw text message (i.e.,[Hadoop logs][hadoop-logs]), and
structured logs are normally JSON records (i.e., [mongoDB logs][mongodb-logs]). [CLP][github-clp],
is a distributed system designed to compress, search, and analyze large-scale log data. It provides
solutions for both unstructured and structured logs, as discussed in its
[2021's OSDI paper][clp-paper] and [2024's OSDI paper][clp-s-paper].

CLP has been deployed in many large-scale production software systems in thousands of cloud servers
and commercial electric vehicles. Throughout the deployment experiences, an interesting issue has
been found. Consider the following log event:
```text
2022-10-10 12:30:02 1563  1827 I AppControl: Removed item: AppOpItem(Op code=1, UID=1000)
```
This is an unstructured log event collected from the Android system on a mobile device. It can be
manually structured in the following way:
```JSON
{
  "timestamp": "2022-10-10 12:30:02",
  "PID": 1563,
  "TID": 1827,
  "priority": "I",
  "tag": "AppControl",
  "record": {
    "action": "Removed item",
    "op_code": 1,
    "UID": 1000
  }
}
```
Intuitively, the structured version makes it easier to query relevant data fields. For example, if
an application wants to query `UID=1000`, it can take advantage of the tree-style key-value pair
structure that JSON format provides. Otherwise, it might need a complicated regular expression to
extract the number from the raw-text log message. Unfortunately, it is impossible to deprecate
unstructured logging infrastructures in any real-world software systems for the following reasons:
- Unstructured logs are more run-time-efficient: it does not introduce overhead of structuring data.
- Legacy issues: real-world software systems use countless software components; some
  may not be compatible with structured logging infrastructure.

Hence, the high-level motivation of our project has been formed: how to improve the analyzability of
unstructured logs to make it as usable as structured logs? The scope of this problem is vast,
and we will focus on one aspect: log parsing. CLP has introduced an innovative way of handling
unstructured logs. The basic idea behind is to find the static text and variables in a raw text log
message, where the static text is like a format string. For instance, the above log event can be
interpreted as the following:
```Python
print(
  f"{timestamp}, {pid}, {tid}, {priority}, {tag}: Removed item: AppOpItem(Op code={op}, UID={uid})"
)
```
`timestamp`, `pid`, `tid`, `priority`, `tag`, `op`, and `uid` are all variables. This provides
some simple data structuring, however, it has a few limitations:
- CLP's heuristic parser cannot parse logs based on user-defined schema. For example,
  `"Removed item"` above may be a variable, but CLP's heuristic parser cannot handle that.
- CLP's heuristic parser cannot parse complicated substrings, i.e., a substring described by the
  regular expression `capture:((?<letterA>a)*)|(((?<letterC>c)|(?<letterD>d)){0,10})`.
- The parsed variables are unnamed. For example, users cannot name the 7th variable to be `"uid"` in
  the above example.

Our project, [log-surgeon-rust][home-page], is designed to improve CLP's parsing features. It is a
safe and high-performant regular expression engine specialized for unstructured logs, allowing users
to extract named variables from raw text log messages efficiently according to user-defined schema.

## Objective and Key Features
The objective of this project is to fill the gap explained in the motivation above in the current
Rust ecosystem. We shall deliver a high-performance and memory-safe log parsing library using Rust.
The project should consist of the core regex engine, the parser, and the user-oriented log parsing
interface.

The core regex engine is designed for high-performance schema matching and variable extraction.
User-defined schemas will be described in regular expressions, and the underlying engine will parse
the schema regular expressions into abstract syntax trees (AST), convert ASTs into non-deterministic
finite automata ([NFA][wiki-nfa]), and merge all NFAs into one large deterministic finite automata
([DFA][wiki-dfa]). This single-DFA design will ensure the execution time is bounded by the length of
the input stream. If time allows, we will even implement [tagged DFA][wiki-tagged-dfa] to make
the schema more powerful.

The parser has two components:
- The schema parser, which is an implementation of [LALR parser][wiki-lalr], parses user-input
schema into regex AST.
- The log parser, which operates similarly to a simple compiler, uses a lexer to process the input
text and emits tokens, and makes decisions based on emitted tokens using the core regex engine.

The log parsing interface will provide user programmatic APIs to:
- Specify inputs (variable schemas) to configure the regex engine
- Feed input stream to the log parser using the configured regex engine
- Retrieve outputs (parsed log events structured according to the user schema) from the parser

[Zhihao Lin][github-zhihao] will be working on the parser implementation.

[Siwei (Louis) He][github-siwei] will be working on the core regex engine implementation.

Both will be working on the log parsing interface.

One will review the other's implementation through GitHub's Pull Request for the purpose of the
correctness and efficiency.

## Tentative Plan and Status
1. **Louis**

| Time                  | Tentative Schedule                          | Status      |
|-----------------------|---------------------------------------------|-------------|
| Oct. 18th ~ Oct. 25th | Complete AST common structs for the project | Done        |
| Oct. 25th ~ Nov. 8th  | Complete NFA structs and research           | On track    |
| Nov. 1st ~ Nov. 8th   | Implement AST to NFA translation            | Not started |
| Nov. 8th ~ Nov. 15th  | Implement AST to NFA translation            | Not started |
| Nov. 15th ~ Nov. 22nd | Complete DFA structs and research           | Not started |
| Nov. 22nd ~ Nov. 29th | Implement NFA to DFA translation            | Not started |
| Nov. 29th ~ Dec. 6th  | Stages integration and final reporting      | Not started |

2. **Zhihao**

| Time                  | Tentative Schedule                                          | Status      |
|-----------------------|-------------------------------------------------------------|-------------|
| Nov. 1st ~ Nov. 15th  | Implement LALR parser for schema parsing and AST generation | Not started |
| Nov. 15th ~ Nov. 29nd | Implement lexer for input stream processing                 | Not started |
| Nov. 29nd ~ Dec. 6th  | Formalize log parsing APIs                                  | Not started |

[clp-paper]: https://www.usenix.org/system/files/osdi21-rodrigues.pdf
[clp-s-paper]: https://www.usenix.org/system/files/osdi24-wang-rui.pdf
[github-clp]: https://github.com/y-scope/clp
[github-siwei]: https://github.com/Louis-He
[github-zhihao]: https://github.com/LinZhihao-723
[hadoop-logs]: https://zenodo.org/records/7114847
[home-page]: https://github.com/Toplogic-Inc/log-surgeon-rust
[mongodb-logs]: https://zenodo.org/records/11075361
[wiki-dfa]: https://en.wikipedia.org/wiki/Deterministic_finite_automaton
[wiki-lalr]: https://en.wikipedia.org/wiki/LALR_parser
[wiki-nfa]: https://en.wikipedia.org/wiki/Nondeterministic_finite_automaton
[wiki-tagged-dfa]: https://en.wikipedia.org/wiki/Tagged_Deterministic_Finite_Automaton
