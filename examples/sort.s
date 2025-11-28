.section .text
    .global _start

#------------------------------------------------------------------------------
# PUTCHAR
#
# Description:
#   Macro to write one byte to the UART address.
#
# Args:
#   reg: The register containing the byte to write
#
# Register Usage:
#   t0: UART Base Address
#------------------------------------------------------------------------------
.macro PUTCHAR reg
    li   t0, 0x10000000                       # t0 <- UART Base Address
    sb   \reg, 0(t0)                          # Store byte to UART
.endm

#------------------------------------------------------------------------------
# _start
#
# Description:
#   Entry point of the program.
#   1. Prints "Unsorted: " and the initial array.
#   2. Performs an in-place Insertion Sort on 'my_array'.
#   3. Prints "Sorted: " and the sorted array.
#   4. Exits via ecall.
#
# Register Usage:
#   s1: Base address of array
#   s2: Array length
#   t0: i (outer loop index)
#   t1: j (inner loop index)
#   t2: key (value to insert)
#   t3: Address offset for arr[i]
#   t4: arr[j]
#   t5: Address offset for arr[j]
#------------------------------------------------------------------------------
_start:
    li    sp, 0x00000000801FF000              # Initialize Stack Pointer

    # ------------------------------------
    # 1. Print "Unsorted: "
    # ------------------------------------
    li a0, 'U'                                # a0 <- 'U'
    PUTCHAR a0                                # Print char
    li a0, 'n'                                # a0 <- 'n'
    PUTCHAR a0                                # Print char
    li a0, 's'                                # a0 <- 's'
    PUTCHAR a0                                # Print char
    li a0, ':'                                # a0 <- ':'
    PUTCHAR a0                                # Print char
    li a0, ' '                                # a0 <- ' '
    PUTCHAR a0                                # Print char

    jal ra, print_array_func                  # Print initial array

    li a0, 10                                 # a0 <- Newline
    PUTCHAR a0                                # Print char

    # ------------------------------------
    # 2. Perform Insertion Sort
    # ------------------------------------
    la   s1, my_array                         # s1 <- Base address of array
    li   s2, 8                                # s2 <- Array length (8)

    li   t0, 1                                # t0 <- 1 (i)

outer_loop:
    bge  t0, s2, sort_done                    # if i >= length goto sort_done

    # key = arr[i]
    slli t3, t0, 3                            # t3 <- i * 8
    add  t3, s1, t3                           # t3 <- base + offset
    ld   t2, 0(t3)                            # t2 <- arr[i] (key)

    addi t1, t0, -1                           # t1 <- i - 1 (j)

inner_loop:
    blt  t1, zero, inner_done                 # if j < 0 goto inner_done

    # load arr[j]
    slli t5, t1, 3                            # t5 <- j * 8
    add  t5, s1, t5                           # t5 <- base + offset
    ld   t4, 0(t5)                            # t4 <- arr[j]

    ble  t4, t2, inner_done                   # if arr[j] <= key goto inner_done

    # arr[j+1] = arr[j]
    sd   t4, 8(t5)                            # Store arr[j] into arr[j+1]

    addi t1, t1, -1                           # t1 <- j - 1
    j    inner_loop                           # goto inner_loop

inner_done:
    # arr[j+1] = key
    addi t5, t1, 1                            # t5 <- j + 1
    slli t5, t5, 3                            # t5 <- (j+1) * 8
    add  t5, s1, t5                           # t5 <- Address of arr[j+1]
    sd   t2, 0(t5)                            # Store key at arr[j+1]

    addi t0, t0, 1                            # t0 <- i + 1
    j    outer_loop                           # goto outer_loop

sort_done:

    # ------------------------------------
    # 3. Print "Sorted: "
    # ------------------------------------
    li a0, 'S'                                # a0 <- 'S'
    PUTCHAR a0                                # Print char
    li a0, 'o'                                # a0 <- 'o'
    PUTCHAR a0                                # Print char
    li a0, 'r'                                # a0 <- 'r'
    PUTCHAR a0                                # Print char
    li a0, 't'                                # a0 <- 't'
    PUTCHAR a0                                # Print char
    li a0, ':'                                # a0 <- ':'
    PUTCHAR a0                                # Print char
    li a0, ' '                                # a0 <- ' '
    PUTCHAR a0                                # Print char

    jal ra, print_array_func                  # Print sorted array

    li a0, 10                                 # a0 <- Newline
    PUTCHAR a0                                # Print char

    # ------------------------------------
    # Exit
    # ------------------------------------
    li a0, 0                                  # a0 <- 0
    li a7, 93                                 # a7 <- 93 (Exit syscall)
    ecall

#------------------------------------------------------------------------------
# print_array_func
#
# Description:
#   Iterates through the global 'my_array' and prints 8 decimal integers
#   separated by spaces.
#
# Args:
#   None (Uses global my_array)
#
# Register Usage:
#   s0: Index counter
#   t5: Base address of my_array
#   t6: Temp (Limit check and offset calculation)
#------------------------------------------------------------------------------
print_array_func:
    addi sp, sp, -16                          # Make room on stack
    sd   ra, 0(sp)                            # Save ra on stack
    sd   s0, 8(sp)                            # Save s0 on stack

    li   s0, 0                                # s0 <- 0 (index)
    la   t5, my_array                         # t5 <- Base address of my_array

pa_loop:
    li   t6, 8                                # t6 <- 8
    bge  s0, t6, pa_done                      # if index >= 8 goto pa_done

    # Load arr[s0]
    slli t6, s0, 3                            # t6 <- index * 8
    add  t6, t5, t6                           # t6 <- base + offset
    ld   a0, 0(t6)                            # a0 <- arr[index]

    jal  ra, print_decimal                    # Print Number

    li   a0, 32                               # a0 <- Space char
    PUTCHAR a0                                # Print char

    addi s0, s0, 1                            # s0 <- index + 1
    j    pa_loop                              # goto pa_loop

pa_done:
    ld   ra, 0(sp)                            # Restore ra from stack
    ld   s0, 8(sp)                            # Restore s0 from stack
    addi sp, sp, 16                           # Restore stack
    ret

#------------------------------------------------------------------------------
# print_decimal
#
# Description:
#   Prints the unsigned integer in a0 to the UART.
#   Converts integer to ASCII string in a local stack buffer, then prints.
#
# Args:
#   a0: The unsigned integer to print
#
# Register Usage:
#   s0: Value to print
#   s1: Length of string
#   t1: Divisor (10)
#   t2: Remainder / Character
#   t3: Pointer to buffer
#------------------------------------------------------------------------------
print_decimal:
    addi sp, sp, -96                          # Make room on stack
    sd   ra, 0(sp)                            # Save ra on stack
    sd   s0, 8(sp)                            # Save s0 on stack
    sd   s1, 16(sp)                           # Save s1 on stack
    sd   s2, 24(sp)                           # Save s2 on stack
    sd   t0, 32(sp)                           # Save t0 on stack
    sd   t1, 40(sp)                           # Save t1 on stack
    sd   t2, 48(sp)                           # Save t2 on stack
    sd   t3, 56(sp)                           # Save t3 on stack

    addi t3, sp, 96                           # t3 <- End of frame
    addi t3, t3, -32                          # t3 <- Buffer end

    add  s0, a0, zero                         # s0 <- Value
    li   s1, 0                                # s1 <- Length

    bne  s0, zero, pd_convert                 # if value != 0 goto pd_convert

    # Handle zero case
    addi t3, t3, -1                           # Decrement buffer pointer
    li   t2, '0'                              # t2 <- '0'
    sb   t2, 0(t3)                            # Store '0' in buffer
    li   s1, 1                                # s1 <- 1
    j    pd_print                             # goto pd_print

pd_convert:
    li   t1, 10                               # t1 <- 10
pd_conv_loop:
    rem  t2, s0, t1                           # t2 <- s0 % 10
    div  s0, s0, t1                           # s0 <- s0 / 10
    addi t2, t2, 48                           # t2 <- Convert to ASCII
    addi t3, t3, -1                           # Decrement buffer pointer
    sb   t2, 0(t3)                            # Store char in buffer
    addi s1, s1, 1                            # s1 <- Length + 1
    bne  s0, zero, pd_conv_loop               # if s0 != 0 goto pd_conv_loop

pd_print:
    beq  s1, zero, pd_done                    # if length == 0 goto pd_done
pd_print_loop:
    lb   a0, 0(t3)                            # a0 <- Char from buffer
    PUTCHAR a0                                # Print char
    addi t3, t3, 1                            # Increment buffer pointer
    addi s1, s1, -1                           # Decrement length
    bne  s1, zero, pd_print_loop              # if length != 0 goto pd_print_loop

pd_done:
    ld   ra, 0(sp)                            # Restore ra from stack
    ld   s0, 8(sp)                            # Restore s0 from stack
    ld   s1, 16(sp)                           # Restore s1 from stack
    ld   s2, 24(sp)                           # Restore s2 from stack
    ld   t0, 32(sp)                           # Restore t0 from stack
    ld   t1, 40(sp)                           # Restore t1 from stack
    ld   t2, 48(sp)                           # Restore t2 from stack
    ld   t3, 56(sp)                           # Restore t3 from stack
    addi sp, sp, 96                           # Restore stack
    ret

#------------------------------------------------------------------------------
# Data Section
#------------------------------------------------------------------------------
    .balign 8
my_array:
    .dword 12
    .dword 5
    .dword 99
    .dword 1
    .dword 9
    .dword 15
    .dword 2
    .dword 20
