                        .ORIG       x3000
                        LEA         R0, msg
                        TRAP        x22
                        LD          R0, endl
                        TRAP        x21
                        LD          R0, endl
                        TRAP        x21
                        LD          R6, stack_start_addr
                        LEA         R3, text 
                        LEA         R0, fwd
                        TRAP        x22
                        LEA         R0, text
                        TRAP        x22
                        LD          R0, endl
                        TRAP        x21

                        ;HALT
                        LDR         R0, R3, #0
                        JSR         _push
                        LDR         R0, R3, #1
                        JSR         _push
                        LDR         R0, R3, #2
                        JSR         _push
                        LDR         R0, R3, #3
                        JSR         _push
                        LDR         R0, R3, #4
                        JSR         _push
                        LEA         R0, rvrs
                        TRAP        x22
                        JSR         _pop
                        TRAP        x21
                        JSR         _pop
                        TRAP        x21
                        JSR         _pop
                        TRAP        x21
                        JSR         _pop
                        TRAP        x21
                        JSR         _pop
                        TRAP        x21
                        LD          R0, endl
                        TRAP	    x21
                        ;Stack should be empty now, check
                        JSR         _pop
                        ADD         R1, R1, #-1 ;expect R1 == 1 
                        BRz         underflw_test_passed
                        LEA         R0, error_msg
                        TRAP        x22
                        LD          R0, endl
                        TRAP        x21
                        HALT
underflw_test_passed    LEA         R0, stack_underflow_msg
                        TRAP        x22
                        LD          R0, endl
                        TRAP        x21
                        
                        HALT

; Push value in R0 to stack, set R0 to #0 if succesful, #1 on overflow
_push                   ST          R1, save_r1 ; Save regs used for overflow check
                        ST          R2, save_r2
                        ;Check for overflow
                        LD          R2, stack_min_addr
                        NOT         R2, R2                  ; make negative
                        ADD         R2, R2, #1              ;
                        ADD         R2, R6, R2
                        BRzp        _push_continue
                        ; else, overflow
                        AND         R0, R0, #0
                        ADD         R0, R0, #1
                        BRnzp       _push_ret
        _push_continue  ADD         R6, R6, #-1              ;no overflow, push to stack  
                        STR         R0, R6, #0
        _push_ret       LD          R1, save_r1
                        LD          R2, save_r2
                        RET
; Pop value from stack to R0; set R1 to #1 if stack is empty
_pop                    ST          R2, save_r2
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
        _pop_empty      AND         R1, R1, #0
                        ADD         R1, R1, #1
                        AND         R0, R0, #0
                        ;LEA         R0, stack_underflow_msg
                        ;TRAP        x22
        _pop_ret        LD          R2, save_r2
                        RET
endl                    .FILL	    #10                    
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
save_r1                 .BLKW       #1
save_r2                 .BLKW	    #1
    .END
