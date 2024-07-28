.ORIG	x3000
        TRAP    x23
        LEA     R0, MSG
        TRAP x22
        HALT
MSG     .STRINGZ	"Bye!"
.END