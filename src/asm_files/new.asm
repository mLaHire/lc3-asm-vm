            .ORIG        x3000
            LEA         R0, HELLO
            PUTS    
            LD          R0, ENDL
            OUT
            HALT
HELLO       .STRINGZ    "Hello World!"
ENDL        .FILL       #10
            .END