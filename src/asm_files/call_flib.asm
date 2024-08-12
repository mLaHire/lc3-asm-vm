        .ORIG	x3090
        ;LD      R0, $float
        ;JSR     $_push
        JSR     $float
        LEA     R0, msg
        PUTS
        LD      R0, endl
        OUT
        HALT
$float  .IMPORT
$_push  .IMPORT
msg     .STRINGZ "This is from within call_flib.asm"
endl    .FILL #10
        .END