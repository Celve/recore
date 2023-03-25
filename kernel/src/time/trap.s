    .section .text.trap
    .globl _timertrap
    .align 2
_timertrap: 
    csrrw sp, mscratch, sp
    sd t0, 0(sp)
    sd t1, 1*8(sp)
    sd t2, 2*8(sp)

    # setup next timer trigger
    ld t0, 3*8(sp) # address of mtimercmp
    ld t1, 4*8(sp) # timer interval      
    ld t2, 0(t0) # current time
    add t2, t2, t1 # new time
    sd t2, 0(t0) # set new time

    # setup timer interrupt for supervisor 
    li t0, 2
    csrw sip, t0
    # csrrs zero, mip, t0

    # restore registers
    ld t0, 0(sp)
    ld t1, 1*8(sp)
    ld t2, 2*8(sp)
    csrrw sp, mscratch, sp
    
    mret
