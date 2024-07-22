            .ORIG x3000
            LEA R1, A
            LDR R0, R1, #0
PRINT       TRAP x21
            ADD R1, R1, #1
            LDR R0, R1, #0
            BRz EXIT
            BRnzp PRINT
EXIT        LEA R1, N
            LDR R0, R1, #0
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