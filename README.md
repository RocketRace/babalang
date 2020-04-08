# What is Babalang?

Babalang is an object-oriented, Turing-complete esoteric programming language inspired by the rule 
system within the (also Turing-complete) indie game Baba Is You. The language is based around statements 
(akin to rules in Baba Is You), each with a subject, verb, one or more targets, and an optional condition. 
The language supports variables, loops, functions, structs, and basic IO operations. 

A full language specification is available at the respective [esolangs.org page](https://esolangs.org/wiki/Babalang).

# The Babalang interpreter

## Compiling

Ensure you have Rust on your machine.

After cloning into this repository, run:

`rustc src/main.rs -Oo babalang`

or if you have Cargo, 

`cargo build --release`

## Running

After creating a file with Babalang source code, run:

`babalang path_to_source_file`

to execute your program.
