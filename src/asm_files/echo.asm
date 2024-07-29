.ORIG   x3000
START       LDI     R1, KBSR
            BRzp    START
            LDI     R0, KBDR
ECHO        TRAP x21
            ;LD      R0, Newline
            ;TRAP x21
            ADD R4, R0, #-10
            BRz EXIT
            BRnzp START
EXIT        HALT
.END
KBSR        .FILL   xFE00
KBDR        .FILL   xFE02
DSR         .FILL   xFE04
DDR         .FILL	xFE06
UPPER       .FILL   #-32
Newline     .FILL   #10