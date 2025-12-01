#ifndef DRIVERS_H
#define DRIVERS_H

#include <stdint.h>

#define UART_BASE 0x10000000
#define DISK_BASE 0x90000000

void uart_putc(char c);
char uart_getc(void);
uint8_t *disk_get_base(void);

#endif
