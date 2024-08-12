;   Service routine to write NULL-term string to console. (From Patt & Patel 2005:236)
;   PUTS (TRAP x22)
;   Input: R0 is a pointer to the string to print
                    .ORIG   x0450
                    ST	    R7,	SaveR7
                    ST      R0, SaveR0
                    ST      R1, SaveR1
                    ST	    R3,	SaveR3
;
;   Loop through each char in the array
;
Loop                LDR     R1, R0, #0  ; Retrieve the char(s)
                    BRz     Return      ; If char is 0, done
L2                  LDI     R3, DSR
                    BRzp    L2
                    STI     R1, DDR     ; Write char
                    ADD     R0, R0, #1  ; Increment pointer
                    BRnzp   Loop
;
;   Return from request for service call
Return              LD      R3, SaveR3
                    LD      R1, SaveR1
                    LD      R0, SaveR0
                    ;LD      R7, SaveR7
                    RET
;
;   Register locations
DSR                 .FILL   xFE04
DDR                 .FILL	xFE06
SaveR0              .FILL   x0000
SaveR1              .FILL   x0000
SaveR3              .FILL   x0000
SaveR7              .FILL   x0000
                    .END