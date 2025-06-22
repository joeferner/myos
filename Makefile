# $@ = target file
# $< = first dependency
# $^ = all dependencies

# First rule is the one executed when no parameters are fed to the Makefile
all: build/os-image.bin

# Notice how dependencies are built as needed
build/kernel.bin: build/kernel_entry.o build/kernel.o
	i386-elf-ld -o $@ -Ttext 0x1000 $^ --oformat binary

build/kernel_entry.o: src/kernel_entry.asm
	nasm $< -f elf -o $@

build/kernel.o: src/kernel.c
	i386-elf-gcc -ffreestanding -c $< -o $@

# Rule to disassemble the kernel - may be useful to debug
build/kernel.dis: build/kernel.bin
	ndisasm -b 32 $< > build/$@

build/bootsect.bin: src/bootsect.asm
	nasm $< -f bin -o $@

build/os-image.bin: build/bootsect.bin build/kernel.bin
	cat $^ > $@

clean:
	rm *.bin *.o *.dis
