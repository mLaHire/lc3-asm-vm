                .ORIG	    x3000
                LEA         R1, M100
                LDR         R0, R1, #0
                ADD         R2, R2, R0
                ADD         R1, R1, #1
                LDR         R0, R1, #0
                ADD         R2, R2, R0
                HALT
                .END
M100            .FILL	    #-100
M10             .FILL	    #-10