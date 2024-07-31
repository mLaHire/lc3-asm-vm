		.ORIG	x3000			
L1 		LEA 		R1, L1
		AND 		R2, R2,	x0
		ADD 		R2, R2, x2
		LD 			R3, P1
L2 		LDR 		R0, R1, xC
		TRAP 		x21
		ADD 		R3, R3, #-1
		BRz 		GLUE
		ADD 		R1, R1, R2
		BRnzp 			L2
GLUE 	HALT
P1		.FILL 		xB
P2		.STRINGZ 	"Abcdefghijklmnop";"HBoeoakteSmtHaotren!s"
		.END 
