            .ORIG       x3000   
            TRAP        x23
            ST          R0, Input
            LEA         R0, MSG
            TRAP        x22
            LD          R0, INPU
            ADD         R0, #10000
            TRAP        x21
            HALT
            .END
MSG         .STRINGZ    "You said: "
NEWLINE     .FILL       #10
INPUT       .BLKW       #10