        .ORIG x3000
        LD      R0, A
        LD      R1, B
L0      ADD     R2, R2, R1
        ADD     R0, R0, #-1
        BRp     L0
        ST      R2, VALUE
        ;LD R0, ANS
        ;LD R3, TOASCII
        ;ADD R0, R0, R3
        ;TRAP x21
        AND     R2, R2, #0
        AND     R1, R1, #0
        AND     R2, R2, #0
                
                IS_NEGATIVE     LD          R5, value
                BRzp        POSITIVE
                NOT         R5, R5
                ADD         R5, R5, #1
                ST          R5, VALUE
                LEA         R0, MINUS
                TRAP        x22
POSITIVE        LEA          R4, M10k
;;              First, check for 100s
AGAIN           LDR	        R1, R4, #0
SUBTR_100       ADD         R2, R2, #1
                ADD         R5, R5, R1
                BRp         SUBTR_100
                BRz         skip        ;if the number is exact
                ADD         R2, R2, #-1
 skip           ST          R2, ANS_100
        LOOP    ADD         R3, R3, R1
                ADD         R2, R2, #-1
                BRp         LOOP
                BRz         DONE
                LDR         R6, R4, #5
                ADD         R3, R6, R3
                ;
        DONE    LD          R5, value
                ADD         R5, R5, R3
                ST          R5, value
        END     LD          R0, ANS_100
                LD          R6, ASCII
                ;STR         R0, R4, #10
                ADD         R0, R0, R6
                LD          R6, IS_NUMBER
                ADD         R6, R0, R6
                BRn         DONT_PRINT
                LD          R1, NON_ZERO
                ADD         R6, R6, R1
                BRz         DONT_PRINT
                ADD         R6, R6, #1
                ST          R6, NON_ZERO
                TRAP        x21
                ;BRnzp       SKIP_ZERO
                ;
                  ;LD          R0, ASCII
                ;TRAP        x21

DONT_PRINT      AND             R2, R2, #0
                AND             R3, R3, #0
                ADD             R4, R4, #1
                ADD             R5, R5, #0
                BRnz            EXIT
                BRnzp           AGAIN
EXIT            LEA             R6, C1
                NOT             R6, R6
                ADD             R6, R6, #1
                ADD             R2, R4, R6 ;Find difference between power and 10^1
                BRp             ESC
L2              LD              R0, ASCII
                TRAP            x21
                ADD             R2, R2, #1
                BRnz            L2
ESC              HALT
A       .FILL #921
B       .FILL #13
ANS       .FILL #0
TOASCII         .FILL #48
TONUMBER        .FILL #-48
M10k            .FILL	    #-10000
M1k             .FILL	    #-1000
C100            .FILL	    #-100
M10             .FILL	    #-10
C1              .FILL	    #-1
PLUS_10k        .FILL	    #10000
PLUS_1000       .FILL	    #1000
PLUS_100        .FILL	    #100
PLUS_10         .FILL	    #10
PLUS_1          .FILL       #1
ANS_10000       .FILL	    #0
ANS_1000        .FILL       #0
ANS_100         .FILL       #0
ANS_10           .FILL      #0
ANS_1            .FILL      #0
;ZERO            .FILL	    #0
ASCII           .FILL	    #48
IS_NUMBER       .FILL	    #-48
NON_ZERO        .FILL	    #0
VALUE           .FILL	    #0
MINUS           .STRINGZ	"-"
MSG             .STRINGZ  "3x16="
.END