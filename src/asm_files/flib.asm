                    .ORIG       x3020
                    LEA         R7, static_exit
$float  .EXPORT     ST          R7, SAVE_R7
                    LEA         R0, msg
                    PUTS
                    LD          R0, endl_
                    OUT
                    LD          R7, SAVE_R7
                    RET
        static_exit HALT
SAVE_R7             .BLKW	    1
msg_                 .STRINGZ	"This is from inside $float, in flib."
endl_                .FILL	    #10
                    .END