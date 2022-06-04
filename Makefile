include Makefile.in

.PHONY: run build fs clean sdcard

all: build 

build:
	cd os \
	&& make BOARD=k210 \
	&& cp $(KERNEL_BIN) ../os.bin

run:
	cd os && make run BOARD=k210

clean:
	rm ./user/target -rf
	rm os.bin \
	&& cd os \
	&& make clean 

vfs: 
	sudo ./makefs.sh

fs: $(FS_IMG)
	./makesd.sh $(SDCARD) $(TMP_MOUNT) $(USER_BIN)
