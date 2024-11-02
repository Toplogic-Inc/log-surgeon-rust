# log-surgeon: A performant log parsing library

Project Link: [Homepage][home-page]

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
print(f"{timestamp}, {pid}, {tid}, {priority}, {tag}: Removed item: AppOpItem(Op code={op_code}, UID={uid})")
```
`timestamp`, `pid`, `tid`, `priority`, `tag`, `op_code`, and `uid` are all variables. This provides
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
The objective of this project is to fill the gap in the Rust ecosystem for a high-performance log parsing library 
specifically designed for multi-schema matching. There are several key features or stages in the log parsing process: Tokenization,
Parsing to AST, Non-deterministic Finite Automata (NFA) construction, NFA to DFA conversion, and final result reporting.

- Tokenization: This is the process of breaking a raw text log message into a sequence of tokens.
- Parsing to AST: This is the process of converting the tokenized log message into an Abstract Syntax Tree (AST). AST is a tree
that is used to represent the syntactic structure of the log message which is easier for the following stages to process than
a raw text string.
- NFA construction: This is the process of constructing a Non-deterministic Finite Automata (NFA) from the AST. Converting from AST 
to NFA is the first step in the process of processing a regular expression.
- NFA to DFA conversion: This is the process of converting the NFA to a Deterministic Finite Automata (DFA). Since NFA
is more expensive to simulate than DFA, this stage is crucial and critical for performance.
- Final result reporting: This is the process of reporting the final result of the log parsing process.

[Zhihao Lin][github-zhihao] will be working on the Tokenization and Parsing to AST stages.

[Siwei (Louis) He][github-siwei] will be working on the Non-deterministic Finite Automata (NFA) construction,
and the NFA to DFA conversion stage. 

There will be integration among the stages to ensure the final result is correct and efficient. The integration work will be distributed
between the two team members.


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

| Time                  | Tentative Schedule | Status      | 
|-----------------------|--------------------|-------------|
| Nov. 1st ~ Nov. 8th   |                    | Not started |
| Nov. 8th ~ Nov. 15th  |                    | Not started |
| Nov. 15th ~ Nov. 22nd |                    | Not started |
| Nov. 22nd ~ Nov. 29th |                    | Not started |
| Nov. 29th ~ Dec. 6th  |                    | Not started |

[clp-paper]: https://www.usenix.org/system/files/osdi21-rodrigues.pdf
[clp-s-paper]: https://www.usenix.org/system/files/osdi24-wang-rui.pdf
[github-clp]: https://github.com/y-scope/clp
[github-siwei]: https://github.com/Louis-He
[github-zhihao]: https://github.com/LinZhihao-723
[hadoop-logs]: https://zenodo.org/records/7114847
[home-page]: https://github.com/Toplogic-Inc/log-surgeon-rust
[mongodb-logs]: https://zenodo.org/records/11075361
