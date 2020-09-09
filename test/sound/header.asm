; Execution after boot begins at $100.
; At $104 the magic header with meta data about the cartridge
; is stored, including the Nintendo logotype. The header content
; is ignored as it is fixed by the "rgbfix" command of RGBDS.
; So we have 4 bytes available, and we use that space to disable
; interrupts (DI, 1 byte) and jump to the Start address (JP, 3 bytes).
SECTION "Header", ROM0[$100]
EntryPoint:
    di          ; Disable interrupts
    jp  Start
REPT $150 - $104    ; Fill header with zeros. Will be updated by "rgbfix".
    db  0
ENDR
