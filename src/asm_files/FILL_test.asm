;
;
;               Test whether .FILL directives work without a label,
;               By simulating 'Z  .STRINGZ "123"
;
                .ORIG           x3000
                LEA             R0, Z
                TRAP            x22
                HALT
                .END
Z               .FILL	        #49
                .FILL	        #50
                .FILL	        #51
                .FILL	        #0