.ORIG x000
        LD R0, CONST
SUB     ADD R0, R0, #-1
        BRp SUB
EXIT    HALT
.END
        CONST .FILL #500