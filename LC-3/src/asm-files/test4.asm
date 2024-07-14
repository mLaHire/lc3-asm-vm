        .ORIG x000
       ADD R1, R1, #15 
       ADD R1, R1, #-5
 LOOP   
        BRnz EXIT
        BRnzp LOOP
 EXIT   HALT
        .END