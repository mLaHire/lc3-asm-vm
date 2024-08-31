                        .ORIG       x2F00
                        HALT
$_init_stack .EXPORT    LD          R6, stack_start_addr
                        LD          R5, stack_start_addr
                        RET
underflw_test_passed    LEA         R0, stack_underflow_msg
                        TRAP        x22
                        LD          R0, endl
                        TRAP        x21                   
; Push value in R0 to stack, set R5 to #1 on overflow
push!   .EXPORT         ST          R1, save_r1 ; Save regs used for overflow check
                        ST          R2, save_r2
                        ;Check for overflow
                        LD          R2, stack_min_addr
                        NOT         R2, R2                  ; make negative
                        ADD         R2, R2, #1              ;
                        ADD         R2, R6, R2
                        BRzp        _push_continue
                        ; else, overflow
                        AND         R5, R5, #0
                        ADD         R5, R5, #1
                        BRnzp       _push_ret
        _push_continue  ADD         R6, R6, #-1              ;no overflow, push to stack  
                        STR         R0, R6, #0
        _push_ret       LD          R1, save_r1
                        LD          R2, save_r2
                        RET
; Pop value from stack to R0; set R5 to #1 if stack is empty
pop!   .EXPORT          ST          R2, save_r2
                        ST          R1, save_r1
                        ; Check for underflow
                        LD          R2, stack_start_addr
                        ADD         R2, R2, #-1
                        NOT         R2, R2
                        ADD         R2, R2, #1
                       ; ADD         R2, R2, #1
                        ADD         R2, R2, R6
                        BRp        _pop_empty
                        LDR         R0, R6, #0
                        ADD         R6, R6, #1
                        AND         R1, R1, #0
                        BRnzp       _pop_ret
        _pop_empty      AND         R5, R5, #0
                        ADD         R5, R5, #1
                        AND         R0, R0, #0
                        ;LEA         R0, stack_underflow_msg
                        ;TRAP        x22
        _pop_ret        LD          R2, save_r2
                        LD          R1, save_r1
                        RET
stack_start_addr        .FILL       x4000
stack_min_addr          .FILL	    x3FF1
stack_max_capacity      .FILL       xF
stack_overflow_msg      .STRINGZ       "Stack overflow."
stack_underflow_msg     .STRINGZ	    "[OK] Stack is empty, underflow detected."
error_msg               .STRINGZ	"[ERR] Error, pop did not detect underflow."
msg                     .STRINGZ	"*Stack demo - reversing strings*"
text                    .STRINGZ	"FILO"
fwd                     .STRINGZ	"Forwards: "
rvrs                    .STRINGZ	"Reverse: "
endl                    .FILL	    #10
save_r1                 .BLKW       #1
save_r2                 .BLKW	    #1
    .END
