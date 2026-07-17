.intel_syntax noprefix

.global _start
_start:
    mov rdi, qword ptr [rsp]
    lea rsi, [rsp + 8]
    and rsp, -16
    call main
    mov rdi, rax
    call exit

#define SYS_exit 60
.global exit
exit:
    mov eax, SYS_exit
    syscall
    ud2

#define SYS_read 0
.global read
read:
    mov eax, SYS_read
    syscall
    ret

#define SYS_write 1
.global write
write:
    mov eax, SYS_write
    syscall
    ret

#define SYS_mmap 9
.global mmap
mmap:
    mov eax, SYS_mmap
    mov r10, rcx
    syscall
    ret

#define SYS_mprotect 10
.global mprotect
mprotect:
    mov eax, SYS_mprotect
    syscall
    ret

#define SYS_open 2
.global open
open:
    mov eax, SYS_open
    syscall
    ret
