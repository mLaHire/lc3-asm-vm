TODO: 

1. **Refactor to separate `ASSEMBLER`, `LINKER/LOADER` amd `VM`                    [ ]**
2. Add config file for all                                                       [ ]
3. **Make separate modules callable via CLI arguments**                              [ ]
4. Rewrite `TOKENIZER`                                                           [ ]
5. Write README and _Getting Started_ Guide                                      [ ]

**Status 12/09/24 21:58**
Made some progress. Separated assembly process significantly,  with the following syntax usable for pre-assembly linking, case sensitivity for labels, logging verbosity and wether to output a symbol file: 

```/lc3-asm-vm assemble examples/new.asm --case-sensitive --verbose-log --no-sym-file --link examples/stacklib.obj.sym```

Gradually replacing old functionality of `main()` with `src/lib.rs.`

**Synopsis of linker process** link .sym file during ASM, then link .obj during loading.
