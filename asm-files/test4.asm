        .ORIG x000
       ADD R1, R1, xE
       ADD R1, R1, #-1
 LOOP   
        BRn EXIT
        BRnzp LOOP
 EXIT   HALT
        .END