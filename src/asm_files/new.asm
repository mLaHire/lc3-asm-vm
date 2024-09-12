            .ORIG        x3000
            LEA         R0, HELLo ;case sensitivity test
            PUTS    
            LD          R0, ENDL
            OUT
            HALT
HELLO       .STRINGZ    "Hello World!"
ENDL        .FILL       #10
            .END