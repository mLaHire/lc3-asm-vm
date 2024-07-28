.ORIG   x0430
                ST      R1, SaveR1   ;R1 used to poll DSR
;
TryWrite        LDI     R1, DSR
                BRzp	TryWrite
WriteIT         STI     R0, DDR
RETURN          LD      R1, SaveR1
                RET
                .END
DSR             .FILL	xFE04
DDR             .FILL	xFE06
SaveR1          .BLKW	#1
                
