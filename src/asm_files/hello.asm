            .ORIG x3000
            LEA R1, A
            LDR R0, R1, #0
PRINT       OUT
            ADD R1, R1, #1
            LDR R0, R1, #0
            BRz EXIT
            BRnZp PRINT
EXIT        LEA R1, N
            LDR R0, R1, #0
            OUT
            LD R0, LETTER
            OUT
            HALT
            .END
A           .STRINGZ "Hello world! Another message..."
OUTPUT         .STRINGZ "MSG"
N          .FILL #10
C           .STRINGZ "C"
LETTER     .FILL #97