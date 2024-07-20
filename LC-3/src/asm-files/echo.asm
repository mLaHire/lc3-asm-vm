.ORIG   x0
START   LDI     R1, KBSR
        BRzp    START
        LDI     R0, KBDR
        ;ADD     R2, R2, #0
        STI     R2, KBSR
        HALT
.END
KBSR    .FILL   xFE00
KBDR    .FILL   xFE02