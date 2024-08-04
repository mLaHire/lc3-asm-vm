        .ORIG x000
        LEA R0, ONE
        TRAP x22
        LEA R0, TWO
        TRAP x22
        HALT
ONE     .STRINGZ "ABC"
TWO     .STRINGZ   "DEF"
BLOCK   .BLKW 5
MORE    .FILL #1
        .FILL #2
        .END