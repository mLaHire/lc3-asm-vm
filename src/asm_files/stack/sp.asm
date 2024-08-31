                    .ORIG       x3000
                    LD          R6, stack_start
                    SP-- 
                    SP--
                    SP++ 
                    SP++
                    SP-- 
                    SP--
                    COPY!       R5, R6 
                    ZERO!       R6
                    ADD         R1, R1, #3
                    SP++ 
                    SP++
                    HALT
stack_start         .FILL       x4000
                    .END