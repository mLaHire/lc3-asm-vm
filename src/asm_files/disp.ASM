                .ORIG       x3000
; INITIALISE    
; R5 = &C100 
; R4 = Offset for R5
; R1 = *(R5), i.e. #-100, then #-10, etc
; R2 = digit_count
; R7 = temp storage
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
                TRAP        x21
                ;BRnzp       SKIP_ZERO
                ;
                  ;LD          R0, ASCII
                ;TRAP        x21
DONT_PRINT      LD          R2, ZERO
                LD          R3, ZERO
                ADD         R4, R4, #1
                ADD         R5, R5, #0
                BRnz        EXIT
                BRnzp       AGAIN
EXIT            LEA         R6, C1
                NOT         R6, R6
                ADD         R6, R6, #1
                ADD         R2, R4, R6 ;Find difference between power and 10^1
                BRzp        ESC
L2              LD          R0, ASCII
                TRAP        x21
                ADD         R2, R2, #1
                BRnz       L2
ESC              HALT
                .END
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
ZERO            .FILL	    #0
ASCII           .FILL	    #48
IS_NUMBER       .FILL	    #-49
VALUE           .FILL	    #500
MINUS           .STRINGZ	"-"