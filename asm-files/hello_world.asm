.ORIG x3000
            LD      R0, MSG
            TRAP    x21
            

HALT
;            .ORIG x0
;CHECK       LDI R0, DSR
;            BRzp CHECK
;            LD R1, MSG
;            STI R1, DDR
;            HALT
.END
T	        .FILL	    #97
DSR         .FILL	    xFE04
DDR         .FILL       xFE06
MSG         .STRINGZ    "Hello world!"