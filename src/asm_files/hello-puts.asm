            .ORIG x3000
            LEA R0, A
            TRAP x22
EXIT        LD R0, N
            TRAP x21
            LD R0, LETTER
            TRAP x21
            HALT
            .END
A           .STRINGZ "Hello world! Another message..."
OUT         .STRINGZ "MSG"
N          .FILL #10
C           .STRINGZ "C"
LETTER     .FILL #97