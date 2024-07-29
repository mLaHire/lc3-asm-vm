.ORIG   x3000
            LEA         R0, HELLO
            TRAP        x22
            LEA         R0, MSG
            TRAP        x22
            LD          R0, N
            TRAP        x21
            LEA         R0, MSG2
            TRAP        x22
            LD          R0, N
            TRAP        x23
            HALT
.END
HELLO       .STRINGZ	"(First)"
MSG         .STRINGZ	"(Second)"
SPACE       .STRINGZ     "<>"
MSG2         .STRINGZ	"(Third)"
N           .FILL	    #10