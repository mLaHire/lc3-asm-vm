        .ORIG x000
        LEA R0, ONE
        TRAP x22
        LEA R0, TWO
        TRAP x22
        HALT
ONE     .STRINGZ "ABC"
TWO     .STRINGZ   "DEF"
        .END