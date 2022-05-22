include Makefile.in

all: build 

build:
	cd os \
	&& make BOARD=k210 \
	&& cp $(KERNEL_BIN) ../os.bin

fs:
	sudo ./mkfs.sh

clean:
	rm os.bin \
	&& cd os \
	&& make clean \

.PHONY: chean fs
