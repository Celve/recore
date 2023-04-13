.altmacro
.macro STORE_SR n
    sd s\n, (\n+2)*8(a0)    
.endm
.macro LOAD_SR n
    ld s\n, (\n+2)*8(a1)
.endm

    .globl _switch
    .section .text
_switch:    
    # store task context
    sd ra, 0*8(a0)
    sd sp, 1*8(a0)
    .set n, 0
    .rept 12
        STORE_SR %n
        .set n, n+1
    .endr

    # load task context
    ld ra, 0*8(a1)
    ld sp, 1*8(a1)
    .set n, 0
    .rept 12
        LOAD_SR %n
        .set n, n+1
    .endr
    ret