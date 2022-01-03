; Based on this tutorial:
; https://eldred.fr/gb-asm-tutorial/hello-world.html

include "hardware.inc"
include "header.asm"
include "fun.asm"

; After booting, the header will disable interrupts and then
; jump to the "Start" address, so this is the real entry point
; of the program.
section "Main", ROM0
Start:
    ; Turn off LCD
    call DisableLCD

    ; With LCD off, VRAM is accessible. Copy font to VRAM:
    ld  hl, $9000
    ld  de, FontTiles
    ld  bc, FontTilesEnd - FontTiles
    call MemCopy

    ; Copy string to tile pointer area of VRAM
    ld  hl, $9800
    ld  de, HelloWorldStr
    call StringCopy

    ; Init display registers
    ld  a, %11100100
    ld  [rBGP], a

    xor a
    ld [rSCY], a
    ld [rSCX], a

    ; Shut sound down
    ld [rNR52], a

    ; Turn screen on, display background
    ld a, %10000001
    ld [rLCDC], a

.lockup:
    jr .lockup

SECTION "Font", ROM0
FontTiles:
INCBIN "font.chr"
FontTilesEnd:

SECTION "Hello World string", ROM0
HelloWorldStr:
    db "12345678901234567890", 0

SECTION "String number two", ROM0
SecondStr:
    db "String two!", 0