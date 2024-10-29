# log-surgeon: A performant log parsing library

Project Link: [Homepage](https://github.com/Toplogic-Inc/log-surgeon-rust)

## Introduction

`log-surgeon` is a library for high-performance parsing of unstructured text
logs implemented using Rust. 


## Motivation
Rust, like many other programming languages, has standard libraries for matching regular expression. However, there is
no high-performance log parsing library specifically designed for multi-schema matching. If you have a long text log, and 
you want to extract structured data from it using multiple schemas, there is no library that can do this efficiently. You
need to call the regular expression matching function n times for n schemas which is inefficient. This project aims to fill this
gap by doing one pass over the log and extracting all the structured data using multiple schemas.

```
Add some reference here to the paper that talks about the problem
Convince the reader about the importance of the problem
```


## Objective and Key Features
The objective of this project is to fill the gap in the Rust ecosystem for a high-performance log parsing library 
specifically designed for multi-schema matching. There are several key features or stages in the log parsing process: Tokenization,
Parsing to AST, Non-deterministic Finite Automata (NFA) construction, NFA to DFA conversion, and final result reporting.

[Zhihao Lin](https://github.com/LinZhihao-723) will be working on the Tokenization and Parsing to AST stages.

[Siwei (Louis) He](https://github.com/Louis-He) will be working on the Non-deterministic Finite Automata (NFA) construction,
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