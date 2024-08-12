.ORIG           x0430
                ST      R1, SaveR1   ;R1 used to poll DSR
                ST      R7, SaveR7
TryWrite        LDI     R1, DSR
                BRzp	TryWrite
WriteIT         STI     R0, DDR
RETURN          LD      R1, SaveR1
                LD      R7, SaveR7
                RET
                .END
DSR             .FILL	xFE04
DDR             .FILL	xFE06
SaveR1          .BLKW	#1
SaveR7          .BLKW	#1
                
