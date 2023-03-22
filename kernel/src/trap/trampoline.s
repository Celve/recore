.altmacro
.macro STORE_REG n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_REG n
    ld x\n, \n*8(sp)
.endm 

    .section .text.trampoline
    .global _alltraps
    .global _restore
    .align 2
_alltraps:
    csrrw sp, sscratch, sp
    sd ra, 1*8(sp) # store ra of user
    sd gp, 3*8(sp) # store gp of user
    .set n, 5
    .rept 27
        STORE_REG %n
        .set n, n+1
    .endr
    csrr t0, sscratch
    sd t0, 2*8(sp) # store sp of user
    csrw sscratch, sp # restore modifid sscratch
    csrr t0, sepc
    csrr t1, sstatus 
    sd t0, 32*8(sp) # store sepc of user
    sd t1, 33*8(sp) # store sstatus of user
    ld t0, 36*8(sp) # load satp of kernel 
    ld t1, 35*8(sp) # load pc of kernel
    ld sp, 34*8(sp) # load sp of kernel
    csrw satp, t0
    sfence.vma
    jr t1

_restore: 
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    ld t0, 32*8(sp) 
    ld t1, 33*8(sp)
    csrw sepc, t0
    csrw sstatus, t1
    ld ra, 1*8(sp)
    ld gp, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_REG %n
        .set n, n+1
    .endr
    ld sp, 2*8(sp)
    sret