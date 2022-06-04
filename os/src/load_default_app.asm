    .section .data
    .align 3
    .global sinitproc
    .global einitproc
    .global susershell
    .global eusershell

sinitproc:
    .incbin "../default_app/target/riscv64gc-unknown-none-elf/release/initproc"
einitproc:

    .section .data
    .align 3
susershell:
    .incbin "../default_app/target/riscv64gc-unknown-none-elf/release/user_shell"
eusershell:
