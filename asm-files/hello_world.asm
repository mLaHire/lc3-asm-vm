.ORIG	x3000
    LD R0, A
    TRAP x21
    LD R0, N
    TRAP x21
    HALT
.END
A   .FILL #97
N   .FILL #10