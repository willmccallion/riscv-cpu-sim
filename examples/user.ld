/* examples/user.ld */
ENTRY(_start)
SECTIONS {
  /* Place the executable code first, at address 0x0 */
  . = 0x0;
  .text : { *(.text .text.*) }
  .rodata : { *(.rodata .rodata.*) }
  .data : { *(.data .data.*) }
  .bss : { *(.bss .bss.* COMMON) }
}
