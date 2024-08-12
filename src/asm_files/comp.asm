.ORIG x3000
INIT_CODE
LEA R6, #-1
ADD R5, R6, #0
ADD R6, R6, R6
ADD R6, R6, R6
ADD R6, R6, R5
ADD R6, R6, #-1
ADD R5, R5, R5
ADD R5, R6, #0
LD R4, GLOBAL_DATA_POINTER
LD R7, GLOBAL_MAIN_POINTER
JSRR R7
HALT

GLOBAL_DATA_POINTER .FILL GLOBAL_DATA_START
GLOBAL_MAIN_POINTER .FILL main
;;;;;;;;;;;;;;;;;;;;;;;;;;;;side;;;;;;;;;;;;;;;;;;;;;;;;;;;;
lc3_side
ADD R6, R6, #-2
STR R7, R6, #0
ADD R6, R6, #-1
STR R5, R6, #0
ADD R5, R6, #-1

ADD R6, R6, #-1
lc3_L1_hello
STR R7, R5, #3
ADD R6, R5, #1
LDR R5, R6, #0
ADD R6, R6, #1
LDR R7, R6, #0
ADD R6, R6, #1
RET

;;;;;;;;;;;;;;;;;;;;;;;;;;;;main;;;;;;;;;;;;;;;;;;;;;;;;;;;;
main
ADD R6, R6, #-2
STR R7, R6, #0
ADD R6, R6, #-1
STR R5, R6, #0
ADD R5, R6, #-1

ADD R6, R6, #-1
ADD R0, R4, #0
LDR R0, R0, #0
jsrr R0
ADD R7, R4, #3
ldr R7, R7, #0
lc3_L2_hello
STR R7, R5, #3
ADD R6, R5, #1
LDR R5, R6, #0
ADD R6, R6, #1
LDR R7, R6, #0
ADD R6, R6, #1
RET


GLOBAL_DATA_START
side .FILL lc3_side
L1_hello .FILL lc3_L1_hello
L2_hello .FILL lc3_L2_hello
L3_hello .FILL #0
.END
