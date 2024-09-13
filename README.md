**LC-3 assembler and virtual machine**
This project provides an assembler and virtual machine for the educational _LC-3_ 16-bit archictechture [1]. 

This implementation covers most, but not _all_ of the specification [2]. Including,

1. Branch instructions
2. Arithmetic instructions
3. Trap routines for:
   a. Polling based input and output

This implementation also includes some of the extensions described here [3], specifically pseudo-ops [4] (`ZERO!`, `SET_COND!` and `COPY!`) as well as the use of the `RES` opcode for built-in `POP!` and `PUSH!`.

**Assembler** 
The _assembler_ produces `.obj` and `.sym` files, which can then be fed into the _virtual machine_. 
The `.obj` files produced appear to be compatible with some other virtual machines [5][6].

The current CLI arguments supported are 

``` --verbose-log : outputs internal details about assembly process ```
``` --case-sensitivive : makes labels case sensitive, closer to original spec [1]```
``` --no-sym-file : no <file>.obj.asm file is generated```

**Linker: overview** 
This implementation introduces a *linker* to LC-3 assembly. 

Currently the linker operates in two stages: (1) it resolves external addressess using a pre-existing `.sym` listing, 
(2) when a `.obj` file is loaded into the virtual machine, it can be loaded alongside other '.obj' files which contain the required addresses. 

```lc3-asm-vm asm lib.asm``` 

produces `lib.obj` and `lib.obj.sym.` 

```lc3-asm-vm prog.asm --link lib.obj.sym``` 

produces `prog.obj` with the correct external addresses. Then, at runtime [7], call

```lc3-asm-vm prog.obj lib.obj``` 

to start a virtual machine instance, load both object files into the memomory and set the program counter to the start address of prog.obj. 

***Linker: assembly syntax***
In assembly, labels are marked for import/export by putting the `.IMPORT` and `.EXPORT` directives immediately after the label.

E.g. in `prog.asm`

```FUNCTION .IMPORT```

and `lib.asm`

```FUNCTION .EXPORT```



[1] Designed by Y. N. Patt & S. Patel, first described in _Introduction to Computing Systems_ Patt & Patel (2000).
[2] Interrupt based I/O, the `RTI` instruction and Program Priority Levels are not currently implemented.
[3] https://www.cs.colostate.edu/~fsieker/TestSemester/assignments/LC3CSU/doc/index.html
[4] With different conventions/terms.
[5] e.g. https://wchargin.com/lc3web/
[6] But where POP! and PUSH! are used the resulting `.obj` file will lead to reserved opcode exceptions.
[7] Note that no error is raised if the user _fails_ to load an external object file, which is a disadvantage of this particular approach 
