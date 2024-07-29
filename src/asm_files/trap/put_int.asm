;           Program to display the value of the number contained in R6 as a string            
                .ORIG       x0
                LEA         R5, CONST_10 ;R5 = pointer to 10k
                LD          R6, INT
                ADD         R3, R3, #-3 ;Power counter 
START           LDR         R1, R5, #0 ;Set R1 to the current power of #10 (-10^
INNER_LOOP      ADD         R2, R2, #1 ;Increment local counter
                ADD         R7, R6, R1
                BRn         RESET
                ST          R7, BLOCK
                LD          R6, BLOCK
                BRp         INNER_LOOP
DISPLAY_DIGIT   LD          R4, TOASCII
                LD          R0, ZERO
                ADD         R0, R2, R4
                TRAP        x21
                NOT         R2, R2
                ADD         R2, R2, R2
                ADD         R6, R2, #0
RESET           LD          R2, ZERO
                ADD         R5, R5, #-1
                ADD         R3, R3, #1
                BRnz        START
                HALT
                .END
ZERO            .FILL       #0
INT             .FILL	    #25
CONST_1         .FILL	    #-1
CONST_10        .FILL	    #-10
CONST_100       .FILL	    #-100
CONST_1K        .FILL	    #-1000
TOASCII         .FILL	    #48
BLOCK           .FILL	    #0
ANS_1           .FILL	    #0
ANS_2           .FILL       #0