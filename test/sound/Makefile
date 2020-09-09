
AS=rgbasm
LINK=rgblink
RGBFIX=rgbfix

default: sound.gb

main.o: main.asm font.chr hardware.inc fun.asm header.asm
	$(AS) main.asm -o main.o

sound.gb: main.o
	$(LINK) -o sound.gb main.o
	$(RGBFIX) sound.gb

sameboy: sound.gb
	open ../../3rd/SameBoy.app sound.gb
