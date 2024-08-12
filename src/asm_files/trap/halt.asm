;Halt service routine, from PATT/PATEL 2005
                .ORIG	xFD70
                ;BRnzp   TEST
                ST      R7, SaveR7
                ST      R1, SaveR1
                ST      R0, SaveR0

                LD	    R0, ENDL
                OUT    
                LEA	    R0, Message
                PUTS
                LD      R0, ENDL
                OUT
;
;   clear bit 15 at xFFFE to stop the machine
;           
                LEA     R2, MCR
                LDI     R1, MCR
                LD      R0, MASK
                AND	    R0, R1, R0
                STI     R0, MCR
;
;       return?
;               
                LD      R1, SaveR1
                LD      R0, SaveR0
                LD      R7, SaveR7
                RET
;
;
;
ENDL            .FILL	x000A
SaveR0          .BLKW   #1
SaveR1          .BLKW   #1
SaveR7          .BLKW   #1
Message         .STRINGZ	"Halting the machine."
MCR             .FILL	xFFFE
MASK            .FILL	x7FFF
                .END