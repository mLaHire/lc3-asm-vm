        .ORIG x000 
        LD R0, A
        LD R1, B
LOOP    ADD R2, R2, R1
        ADD R0, R0, #-1
        BRp LOOP
END     ST R2, ANS
        ;LD R0, ANS
        ;LD R3, TOASCII
        ;ADD R0, R0, R3
        ;TRAP x21
        HALT
        .END
A       .FILL #7999
B       .FILL #51
ANS       .FILL #0
TOASCII         .FILL #48
TONUMBER        .FILL #-48
