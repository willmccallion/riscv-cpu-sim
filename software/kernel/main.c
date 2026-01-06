#include "drivers.h"
#include "fs.h"
#include "kdefs.h"
#include "klib.h"
#include "mm.h"

void print_banner() {
  kprint("\n");
  kprint(ANSI_CYAN "RISC-V MicroKernel v2.2.0" ANSI_RESET "\n");
  kprint("Build: " __DATE__ " " __TIME__ "\n");
  kprint("CPUs: 1 | RAM: 128MB | Arch: rv64im\n\n");

  kprint("[ " ANSI_GREEN "OK" ANSI_RESET " ] Initializing UART...\n");

  // Initialize Physical Memory Manager
  kinit();
  kprint("[ " ANSI_GREEN "OK" ANSI_RESET " ] Physical Memory Manager...\n");

  // Test Allocation to ensure PMM works
  void *p = kalloc();
  if (p) {
    kprint("[ " ANSI_GREEN "OK" ANSI_RESET " ] PMM Test: Alloc at ");
    kprint_hex((uint64_t)p);
    kprint("\n");
    kfree(p);
  } else {
    kprint("[ " ANSI_RED "FAIL" ANSI_RESET " ] PMM Alloc failed!\n");
  }

  kprint("[ " ANSI_GREEN "OK" ANSI_RESET " ] Mounting Virtual Disk...\n");
  kprint("[ " ANSI_GREEN "OK" ANSI_RESET " ] System Ready.\n\n");
}

void kmain() {
  print_banner();
  long last_exit_code = 0;

  while (1) {
    kprint(ANSI_GREEN "root@riscv" ANSI_RESET ":" ANSI_CYAN "~" ANSI_RESET);

    if (last_exit_code != 0) {
      kprint(ANSI_RED " (");
      kprint_long(last_exit_code);
      kprint(")" ANSI_RESET);
      last_exit_code = 0;
    }

    kprint("# ");

    char cmd[32];
    kgets(cmd, 32);

    if (cmd[0] == 0)
      continue;

    if (kstrcmp(cmd, "ls") == 0) {
      fs_ls();
      continue;
    }

    if (kstrcmp(cmd, "help") == 0) {
      kprint("Built-ins: ls, help, clear, exit\n");
      continue;
    }

    if (kstrcmp(cmd, "clear") == 0) {
      kprint("\x1b[2J\x1b[H");
      continue;
    }

    if (kstrcmp(cmd, "exit") == 0) {
      kprint("[" ANSI_GREEN " OK " ANSI_RESET "] System halting.\n");
      // We add NOPs here to flush the pipeline.
      asm volatile("li a7, 93\n"
                   "li a0, 0\n"
                   "nop\n"
                   "nop\n"
                   "nop\n"
                   "nop\n"
                   "ecall");
      while (1)
        ;
    }

    // Try to find file in FS
    struct FileHeader fh;
    if (fs_find(cmd, &fh)) {
      kmemset((void *)RAM_USER_BASE, 0, 0x100000);
      fs_load(&fh, (void *)RAM_USER_BASE);

      long code = switch_to_user(RAM_USER_BASE);

      if (code >= 0 && code <= 255) {
        last_exit_code = code;
      } else {
        kprint("\n" ANSI_RED "[FATAL] Trap Cause: ");
        kprint_hex((uint64_t)code);
        kprint(ANSI_RESET "\n");
        last_exit_code = 139;
      }
    } else {
      kprint("sh: command not found: ");
      kprint(cmd);
      kprint("\n");
      last_exit_code = 127;
    }
  }
}
