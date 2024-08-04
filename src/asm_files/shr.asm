;shr[1](x)
;R0 -> x
;R1 -> offset from F8
;R2 -> value of FN 
;R3 -> result of calcs
;R4 -> SHR[1](x)
                .ORIG	        x3000
                LD              R0, num
                ST              R0, value
                LEA             R0, msg1
                PUTS
                JSR             $f_put_int


                LD              R0, endl
                OUT
                AND             R4, R4, #0
                LD              R0, num
                LEA             R1, f_2to14
check_for_fn    LDR             R2, R1, #0
                BRz             shr_done
                AND             R3, R2, R0 ;R3 <- R0 & F_N
                NOT             R3, R3
                ADD             R3, R3, #1 ;R3 <- -R3

                ADD             R3, R2, R3
                BRp            no_fn            ;i.e. R0 & F_N != FN
                BRnz            fn
    fn          LDR             R2, R1, #1      ;LD F_(N+1) into R2
                ADD             R4, R4, R2      ;Add R4 <- R4 + F_(N+1)
    no_fn       ADD             R1, R1, #1
                BRnzp           check_for_fn
shr_done        AND             R0, R0, #0
                ADD             R0, R0, R4
                ;ST              R0, value_save
                ST              R4, value
                ;HALT
                LEA             R0, msg2
                PUTS
                AND             R6, R6, #0
                ;LD              R0, value_save
              
                JSR             $f_put_int
                HALT
;;
;; DISPLAY NUM

$f_put_int      ST              R7, SaveR7        
                AND             R0, R0, #0
                AND             R1, R1, #0
                AND             R2, R2, #0
                AND             R3, R3, #0 ;;absence Caused bug
               ; AND             R4, R4, #0
                ST              R3, ANS_100
                ST              R3, NON_ZERO
                ;AND             R6, R6, #0 ;;absence Caused bug
IS_NEGATIVE     LD          R5, value
                BRp        POSITIVE
                BRz        NUM_IS_ZERO
                NOT         R5, R5
                ADD         R5, R5, #1
                ST          R5, VALUE
                LD         R0, MINUS
                TRAP        x21
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
                OUT
                ADD             R2, R2, #1
                BRnz            L2
ESC             LD              R7, SaveR7
                RET
NUM_IS_ZERO     LD              R0, TOASCII ;Ascii zero
                OUT 
                LD              R7, SaveR7
                RET
A       .FILL #93
B       .FILL #117
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
                .FILL       #0
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
VALUE_SAVE      .FILL	    #0
MINUS           .STRINGZ	"-"
;f_2to15         .FILL	        x8000      
f_2to14         .FILL	        x4000                
f_8192          .FILL	        x2000
f_4096          .FILL	        x1000
f_2048          .FILL	        x800
f_1024          .FILL	        x400
f_512           .FILL	        x200
f_256           .FILL	        x100
f_128           .FILL	        x80
f_64            .FILL           x40
f_32            .FILL           x20
f_16            .FILL           x10
f_8             .FILL           x8
f_4             .FILL	        x4
f_2             .FILL	        x2
f_1             .FILL	        x1
f_0             .FILL	        x0
f_null           .FILL	        #-1
ascii_int        .FILL	        #48
num              .FILL	        #32145
SaveR7          .BLKW	        1
msg1            .STRINGZ	    "x: "
msg2            .STRINGZ        "Shr(x): "
endl            .FILL	        #10
                .END



