                .ORIG x3000
                JSR	            f
                LEA             R0, msg
                TRAP            x22
                LD              R0, ENDL
                TRAP            x21
                HALT

f               ST              R7, CALLER_ADDR
                LEA             R0, f_msg
                TRAP            x22
                LD              R0, ENDL
                TRAP            x21
                LD              R7, CALLER_ADDR
                RET
f_MSG           .STRINGZ        "This is a function -> f()..."
MSG             .STRINGZ	    "Back from function."
ASCII           .FILL	        #50
ENDL            .FILL           #10
CALLER_ADDR     .BLKW	        #1	        
                .END