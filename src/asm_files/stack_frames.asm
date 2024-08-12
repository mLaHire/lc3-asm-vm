                .ORIG	    x3000
                JSR         $_init_stack
                ADD         R0, R0, #1
                JSR         $_push
                AND         R0, R0, #0
                ADD         R0, R0, #-2
                JSR         $_push
                JSR         $OpMul
                JSR         $_pop
                LD          R1, ASCII_ZERO
                ADD         R0, R0, R1
                OUT     
                HALT
$OpMul          ST          R0, saveR0
                ST          R1, saveR1
                ST          R2, saveR2
                ST          R3, saveR3
                ST          R7, saveR7
                JSR         $_pop
                ST          R0, num1
                JSR         $_pop
                ST          R0, num2

;               Check sign of num1
                LD          R0, num1
                BRp         check_num2
                BRz         exit
                ;if num1 is negative
                NOT         R0, R0
                ADD         R0, R0, #1
                ST          R0, num1
                LD          R0, TRUE
                ST          R0, is_neg_num1

check_num2      LD          R0, num2
                BRp         check_done
                BRz         exit
                ;if num2 is negative
                NOT         R0, R0
                ADD         R0, R0, #1
                ST          R0, num2
                LD          R0, TRUE
                ST          R0, is_neg_num2

                ;begin multiplication
check_done      LD          R1, num1
                LD          R2, num2
                AND         R3, R3, #0      ;R3 <- 0
MUL             ADD         R3, R3, R1
                ADD         R2, R2, #-1
                BRp         MUL
                
;               Check for negative signs
                LD          R1, is_neg_num1
                LD          R2, is_neg_num2

                AND         R0, R0, #0
                ADD         R0, R1, R2
                AND         R1, R0, b1
                BRp         diff_sign
                BRz         same_sign
                AND         R1, R0, b10
                BRp         same_sign
diff_sign       NOT         R3, R3
                ADD         R3, R3, #1
                HALT
same_sign       AND         R0, R0, #0
                ADD         R0, R0, R3
p               JSR         $_push
exit            LD          R0, saveR0
                LD          R1, saveR1
                LD          R2, saveR2
                LD          R3, saveR3
                LD          R7, saveR7
                RET

$_pop           .IMPORT
$_push          .IMPORT
$_init_stack    .IMPORT
num1            .BLKW	    1
num2            .BLKW	    2
saveR0          .BLKW       1
saveR1          .BLKW       1
saveR2          .BLKW       1
saveR3          .BLKW       1
saveR7          .BLKW	    1
is_neg_num1     .FILL       #0
is_neg_num2     .FILL	    #0
TRUE            .FILL	    #1
ASCII_ZERO      .FILL	    #48
                .END