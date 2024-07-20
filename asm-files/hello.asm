            .ORIG x3000
            LEA R1, A
            LDR R0, R1, #0
PRINT       TRAP x21
            ADD R1, R1, #1
            LDR R0, R1, #0
            BRz EXIT
            BRnzp PRINT
EXIT        LEA R1, B
            LDR R0, R1, #0
            TRAP x21
            HALT
            .END
A         .STRINGZ "Hello world!"
B           .FILL #10
C           .STRINGZ "C"
NEWLINE     .FILL #97