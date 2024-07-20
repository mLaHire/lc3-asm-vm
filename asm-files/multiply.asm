        .ORIG x000 
        LD R0, A
        LD R1, B
LOOP    ADD R2, R2, R1
        ADD R0, R0, #-1
        BRp LOOP
END     ST R2, ANS
        LD R0, ANS
        HALT
        .END
;
A       .FILL #7
B       .FILL #0
ANS       .FILL #0