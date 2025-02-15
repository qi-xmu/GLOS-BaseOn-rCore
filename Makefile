include Makefile.in

.PHONY: run build fs vfs clean qemu

all: build 

build:
	cd os \
	&& make BOARD=k210 \
	&& cp $(KERNEL_BIN) ../os.bin

run:
	cd os && make run BOARD=k210

qemu:
	cd os && make run

clean:
	rm ./user/target -rf
	rm os.bin \
	&& cd os \
	&& make clean 

vfs: 
	cd os && make vfs

fs: 
	./user/build-oscomp.sh
	./makesd.sh $(SDCARD) $(TMP_MOUNT) $(USER_BIN)
