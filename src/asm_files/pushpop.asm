             .ORIG       x3000
            LD          R6, stack_start
            LD          R3, ZERO
            PUSH!       R1
            ADD         R0, R0, #5
            AND         R1, R1, #0
            ADD         R1, R1, R0
            ADD         R1, R1, R2
            PUSH!       R1
;----------------------------------------------------------------------
            LEA         R0, text1
            PUTS        
            POP!        R0
            OUT
            LD          R0, ENDL
            OUT
            LEA         R0, text1
            PUTS        
            POP!        R0
            OUT     
            LD          R0, ENDL
            OUT
            HALT
ZERO        .FILL	    #48
ENDL        .FILL	    #10
text1       .STRINGZ	"POP! "
stack_start .FILL	    x4000
$_init_stack    .IMPORT
            .END
            