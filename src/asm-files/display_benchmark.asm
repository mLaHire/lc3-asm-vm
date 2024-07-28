            .ORIG	    x3000
            LD          R1, TIMES
REPEAT      LEA         R0, TEXT
            TRAP        x22
            LD          R0, ENDL
            TRAP        x21
            ADD         R1, R1, #-1
            BRp         REPEAT
            HALT
            .END
TIMES       .FILL	    #10
ENDL        .FILL	    #10
TEXT        .STRINGZ    "This is a long message. How long does it take?"