                .ORIG       x3000
                LEA         R0, $_init_stack
                JSR         $_init_stack
                LEA         R0, endl
                JSR         $_push
                LEA         R0, text2
                JSR         $_push
                LEA         R0, text1
                JSR         $_push
                ;
                JSR         $_pop
                PUTS
                JSR         $_pop
                PUTS        
                JSR         $_pop
                PUTS
                HALT
text1           .STRINGZ	"Hello... "
text2           .STRINGZ    "World!"
endl            .FILL	    #10
                .FILL       #0
$_pop           .IMPORT
$_push          .IMPORT
$_init_stack    .IMPORT
                .END