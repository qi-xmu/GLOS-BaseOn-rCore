    .section .data
    .align 3
    .global sinitproc
    .global einitproc
sinitproc:
    .incbin "../default_app/target/riscv64gc-unknown-none-elf/release/initproc"
einitproc:

    .section .data
    .align 3
    .global stest
    .global etest
stest:
    .incbin "../default_app/target/riscv64gc-unknown-none-elf/release/test_all"
etest:

    .section .data
    .align 3
    .global shello_world
    .global ehello_world
shello_world:
    .incbin "../default_app/target/riscv64gc-unknown-none-elf/release/hello_world"
ehello_world:
