            .ORIG       x3000
            IN
            ST          R0, Input
            LEA         R0, MSG
            PUTS    
            LD          R0, INPUT
            OUT
            HALT
            .END
MSG         .STRINGZ    "You said: "
NEWLINE     .FILL       #10
INPUT       .BLKW       #10 