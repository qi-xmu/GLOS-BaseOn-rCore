# CMAKE generated file: DO NOT EDIT!
# Generated by "Unix Makefiles" Generator, CMake Version 3.16

# Delete rule output on recipe failure.
.DELETE_ON_ERROR:


#=============================================================================
# Special targets provided by cmake.

# Disable implicit rules so canonical targets will work.
.SUFFIXES:


# Remove some rules from gmake that .SUFFIXES does not remove.
SUFFIXES =

.SUFFIXES: .hpux_make_needs_suffix_list


# Suppress display of executed commands.
$(VERBOSE).SILENT:


# A target that is always out of date.
cmake_force:

.PHONY : cmake_force

#=============================================================================
# Set environment variables for the build.

# The shell in which to execute make rules.
SHELL = /bin/sh

# The CMake executable.
CMAKE_COMMAND = /usr/bin/cmake

# The command to remove a file.
RM = /usr/bin/cmake -E remove -f

# Escaping for special characters.
EQUALS = =

# The top-level source directory on which CMake was run.
CMAKE_SOURCE_DIR = /home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user

# The top-level build directory on which CMake was run.
CMAKE_BINARY_DIR = /home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user/build

# Include any dependencies generated for this target.
include CMakeFiles/times.dir/depend.make

# Include the progress variables for this target.
include CMakeFiles/times.dir/progress.make

# Include the compile flags for this target's objects.
include CMakeFiles/times.dir/flags.make

CMakeFiles/times.dir/src/oscomp/times.c.o: CMakeFiles/times.dir/flags.make
CMakeFiles/times.dir/src/oscomp/times.c.o: ../src/oscomp/times.c
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Building C object CMakeFiles/times.dir/src/oscomp/times.c.o"
	riscv64-unknown-elf-gcc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -o CMakeFiles/times.dir/src/oscomp/times.c.o   -c /home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user/src/oscomp/times.c

CMakeFiles/times.dir/src/oscomp/times.c.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing C source to CMakeFiles/times.dir/src/oscomp/times.c.i"
	riscv64-unknown-elf-gcc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -E /home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user/src/oscomp/times.c > CMakeFiles/times.dir/src/oscomp/times.c.i

CMakeFiles/times.dir/src/oscomp/times.c.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling C source to assembly CMakeFiles/times.dir/src/oscomp/times.c.s"
	riscv64-unknown-elf-gcc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -S /home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user/src/oscomp/times.c -o CMakeFiles/times.dir/src/oscomp/times.c.s

# Object files for target times
times_OBJECTS = \
"CMakeFiles/times.dir/src/oscomp/times.c.o"

# External object files for target times
times_EXTERNAL_OBJECTS =

riscv64/times: CMakeFiles/times.dir/src/oscomp/times.c.o
riscv64/times: CMakeFiles/times.dir/build.make
riscv64/times: libulib.a
riscv64/times: CMakeFiles/times.dir/link.txt
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --bold --progress-dir=/home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Linking C executable riscv64/times"
	$(CMAKE_COMMAND) -E cmake_link_script CMakeFiles/times.dir/link.txt --verbose=$(VERBOSE)
	mkdir -p asm
	riscv64-unknown-elf-objdump -d -S /home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user/build/riscv64/times > asm/times.asm
	mkdir -p bin
	riscv64-unknown-elf-objcopy -O binary /home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user/build/riscv64/times bin/times.bin --set-section-flags .bss=alloc,load,contents

# Rule to build all files generated by this target.
CMakeFiles/times.dir/build: riscv64/times

.PHONY : CMakeFiles/times.dir/build

CMakeFiles/times.dir/clean:
	$(CMAKE_COMMAND) -P CMakeFiles/times.dir/cmake_clean.cmake
.PHONY : CMakeFiles/times.dir/clean

CMakeFiles/times.dir/depend:
	cd /home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user/build && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user /home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user /home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user/build /home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user/build /home/qi/Rcore/testsuits-for-oskernel/riscv-syscalls-testing/user/build/CMakeFiles/times.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : CMakeFiles/times.dir/depend

