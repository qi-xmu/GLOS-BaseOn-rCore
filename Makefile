include Makefile.in

.PHONY: run build fs clean

all: build 

build:
	cd os \
	&& make BOARD=k210 \
	&& cp $(KERNEL_BIN) ../os.bin

run:
	cd os && make run BOARD=k210

fs:
	sudo ./mkfs.sh

clean:
	rm ./user/target -rf
	rm os.bin \
	&& cd os \
	&& make clean 

TEST_IMG := riscv-syscalls-testing/user/riscv/fs.img

$(FS_IMG): $(APPS)
	@cd ./user && make build
	@cd ./mkfs && \
	cargo run --release -- \
		-s ../user/target/riscv64gc-unknown-none-elf/release/ \
		-t ../user/target/riscv64gc-unknown-none-elf/release/

$(APPS):
	# 

$(TEST_IMG):$(APPS)
	@./riscv-syscalls-testing/user/build-oscomp.sh
	@cd ./mkfs && cargo run --release -- -s ../user/src/bin/ -t ../$(TEST_IMG)


sdcard: $(TEST_IMG)
	@echo "Are you sure write to $(SDCARD) ? [y/N] " && read ans && [ $${ans:-N} = y ]
	@sudo dd if=/dev/zero of=$(SDCARD) bs=1048576 count=16
	@sudo dd if=$^ of=$(SDCARD)



.PHONY: chean fs sdcard
