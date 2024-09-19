# Overview
This project provides an assembler and virtual machine for the educational _LC-3_ 16-bit archictechture, originally designed by Patt and Patel [^1]. 

## Specification
This implementation covers most, but not _all_ of the specification [^2]. Including,

1. Branch instructions
2. Arithmetic instructions
3. Trap routines for:
   a. Polling based input and output

This implementation also incorporates some of the extensions introduced by F. Sieker [^3], specifically pseudo-ops [^4] (`ZERO!`, `SET_COND!` and `COPY!`) as well as the use of the `RES` opcode for built-in `POP!` and `PUSH!`.

## Assembler 
The _assembler_ produces `.obj` and `.sym` files, which can then be fed into the _virtual machine_. 
The `.obj` files, which are machine code binaries, appear to be compatible with some other virtual machines [^5][^6].

The assembler, invoked with `lc3-asm-vm asm` currently supports the following CLI flags:

``` --verbose-log : outputs internal details about assembly process ```

``` --case-sensitivive : makes labels case sensitive, closer to original spec```

``` --no-sym-file : no <file>.obj.asm file is generated```

## Running the VM: Linker/Loader
This implementation introduces a *linker* to LC-3 assembly. 

Currently the linker operates in two stages: (1) it resolves external addressess using a pre-existing `.sym` listing, 
(2) when a `.obj` file is loaded into the virtual machine, it can be loaded alongside other '.obj' files which contain the required addresses. 

```lc3-asm-vm asm lib.asm``` 

produces `lib.obj` and `lib.obj.sym.` 

```lc3-asm-vm prog.asm --link lib.obj.sym``` 

produces `prog.obj` with the correct external addresses. Then, at runtime [^7], call

```lc3-asm-vm prog.obj lib.obj``` 

to start a virtual machine instance, load both object files into the memory and set the program counter to the start address of prog.obj. 

## Linker: assembly syntax
In assembly, labels are marked for import/export by putting the `.IMPORT` and `.EXPORT` directives immediately after the label.

E.g. in `prog.asm`

```FUNCTION .IMPORT```

and `lib.asm`

```FUNCTION .EXPORT ADD R0, R1, #3```


# Getting Started
1. Install Rust.
2. Download or clone the repo.
3. Navigate to the root directory of the install.
4. Run `cargo build`
5. Run `lc3-asm-vm`...
   

###### Example: hello_world.asm 

```
;
;	Hello World! in LC3 assembly.
;
		.ORIG	x3000
		LEA	R0, hello
		PUTS    
		LD	R0, endl
		OUT
		HALT
hello		.STRINGZ "Hello World!"
endl		.FILL	#10
		.END

```


To assemble this program, you would enter (ignoring file paths)

	lc3-asm-vm asm hello_world.asm

Then to run it on the simulator or virtual machine: 

	lc3-asm-vm load hello_world.obj


# Approach
*lc3-asm-vm* is written in Rust, and the only dependency outside of the Rust Standard Library is the `console` crate. However, as it stands, external libraries for, e.g., config file loading and CLI argument parsing would provide more functionality with less effort. 

At the same time, I wrote this project with the goal of learning about CPU/IS architechture, Rust, assembly and assemblers/compilers.  There are design decisions that could be improved on, especially concerning the tokenizer and parser. But the process of finding solutions, even if they are not the *ideal* solution, cements learning in a way that just implementing or leveraging existing solutions almost never can.



[^1]: Designed by Y. N. Patt & S. Patel, first described in _Introduction to Computing Systems_ Patt & Patel (2000).
[^2]: Interrupt based I/O, the `RTI` instruction and Program Priority Levels are not currently implemented.
[^3]: https://www.cs.colostate.edu/~fsieker/TestSemester/assignments/LC3CSU/doc/index.html
[^4]: With different conventions/terms.
[^5]: e.g. https://wchargin.com/lc3web/
[^6]: But where POP! and PUSH! are used the resulting `.obj` file will lead to reserved opcode exceptions.
[^7]: Note that no error is raised if the user _fails_ to load an external object file, which is a disadvantage of this particular approach.
