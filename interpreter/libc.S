.intel_syntax noprefix

#define SYS_exit 60
.global exit
exit:
    mov rax, SYS_exit
    syscall
    ud2

#define SYS_read 0
.global read
read:
    mov rax, SYS_read
    syscall
    ret

#define SYS_write 1
.global write
write:
    mov rax, SYS_write
    syscall
    ret

#define SYS_mmap 9
.global mmap
mmap:
    mov rax, SYS_mmap
    mov r10, rcx
    syscall
    ret

#define SYS_mprotect 10
.global mprotect
mprotect:
    mov rax, SYS_mprotect
    syscall
    ret
