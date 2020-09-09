; Misc functions
section "Functions", ROM0

; Wait for VBlank. If efficiency is important it's better
; to implement this locally to avoid spending extra cycles
; on the RET operation
WaitForVBlank:
    ld  a, [rLY]
    cp  144
    jr  c, WaitForVBlank
    ret

; Disable LCD, for example to allow access to VRAM
; Waits for VBlank and sets LCDC to 0
DisableLCD:
    ld  a, [rLY]
    cp  144
    jr  c, DisableLCD
    xor a
    ld  [rLCDC], a
    ret

; Copy a given number of bytes from one area to another
; @param hl - destination address
; @param de - source address
; @param bc - number of bytes
MemCopy:
    ld  a, [de]     ; Copy value stored at [de] to a
    ld  [hli], a    ; Store value in a to [hl] and increment hl
    inc de          ; Increment source address
    dec bc          ; Decrement number of bytes remaining
    ld  a, b
    or  c           ; Set carry flag if bc is zero
    jr  nz, MemCopy ; If bc is not zero, copy next byte
    ret

; Copy a zero terminated string to other area in memory
; @param hl - destination address
; @param de - source address
StringCopy:
    ld  a, [de]     ; Copy value stored at [de] to a
    ld  [hli], a    ; Store value in a to [hl] and increment hl
    inc de          ; Increment source address
    and a           ; Check if the byte just copied is zero
    jr  nz, StringCopy  ; If not zero, we're not at end of string
    ret
