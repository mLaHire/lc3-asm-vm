        .ORIG x301c
        ST R3, SAVE3
        ST R2, SAVE2
        AND R2, R2, #0
TEST    IN
        BRz TEST
        ADD Rl, R0, #-10
        BRn FINISH
        ADD R1, R0, #-15
        NOT R1, R1
        JMP R0
        BRn FINISH
        HALT
FINISH  ADD R2, R2, #1
        HALT
SAVE3   .FILL x0000
SAVE2   .FILL x0000
        .END 