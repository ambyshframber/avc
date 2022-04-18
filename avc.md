# AVC INSTRUCTION SET ARCHITECTURE

The AJAL VIRTUAL CPU has 6 registers:

`a`: general purpose 1

`b`: general purpose 1

`x`: offset

`s`: status

`pc`: program counter

`sp`: stack pointer

Of these, only `a` can be directly loaded and stored to memory. Other registers must be edited using the register manipulation instructions. `a`, `b`, `x`, and `s` are 8-bit, and the remaining two are 16-bit.

Bit 0 of the status register is the carry or "inverse borrow" flag, similar to the 6502. Bit 1 is the zero flag. Bits 2-7 are currently unused.

## INSTRUCTIONS

### GENERAL

`nop`: No operation.

`hlt`: Halt the processor immediately.

### REGISTER MANIPULATION

`swp`: Swap the values of `a` and `b`.

`tab`: Copy `a` into `b`.

`tax`: Copy `a` into `x`.

`txa`: Copy `x` into `a`.

`inc`: Add 1 to `x`.

`dec`: Subtract 1 from `x`.

### ARITHMETIC

`add`: Store the result of `a` + `b` in `a`. If the carry flag is set, an extra 1 is added. If the value would exceed 255, the carry flag is set in the status register, otherwise it is unset. There is no subtract instruction. Use `not` and `swp` to your advantage.

`lsr`: Shift `a` to the right by 1 bit, multiplying it by 2. If a bit would be shifted "out", the carry flag is set, otherwise it is unset. Likewise, if the carry flag is set, a bit will be shifted "in".

`lsl`: Shift `a` to the left by 1 bit, dividing it by 2. "In" and "out" shifting behaviour is identical to `lsr`.

### BITWISE OPERATIONS

`and`: Store the result of `a` & `b` in `a`.

`not`: Invert the bits of `a`.

`ior`: Inclusive or. Store the result of `a` | `b` in `a`.

`xor`: Exclusive or. Store the result of `a` ^ `b` in `a`.

### CARRY

`sec`: Set the carry flag.

`clc`: Clear the carry flag.

### STACK

`psa`: Push `a` onto the stack.

`ppa`: Pop a value from the stack and transfer it into `a`.

`gsp`: Copy the high byte of the stack pointer into `a`, and the low byte into `b`.

`ssp`: Copy `a` into the high byte of the stack pointer, and `b` into the low byte.

### JUMPS

`jmp`: Unconditional jump.

`jez`: Jump if the zero flag is set.

`jgt`: Jump if `a` is greater than `b`.

`jsr`: Push the value of the program counter to the stack (lo-byte first) and then jump to the specified address.

`rts`: Pop the program counter from the stack (hi-byte first) and jump to it.

### LOADING AND STORING

`lda`: Load the value at the specified address into `a`.

`sta`: Store the value of `a` at the specified address.

### MISC

`out`: Send the value of `a` into the virtual output buffer. By default, this manifests as printing the ASCII character to the terminal.

`get`: Read a byte from the input buffer into `a`. If the buffer is empty, 0 will be written. The buffer is updated every time a newline is sent.

`gbf`: Get the length of the current input buffer and copy it into `a`. If the length is greater than 255, the carry flag is set.

`brk`: Specifies a debugging breakpoint for the simulation.

## ADDRESSING MODES

AVC supports offset and indirect addressing for all instructions that use addressing, as well as literal addressing for `lda` only. It uses big-endian ordering.

By default, the address given in the assembly code is placed after the instruction, and is used directly by the instruction. `lda 0x0300` will cause the processor to load the byte at `0x0300`.

Offset addressing is signified by placing `,x` after the address (for example, `lda 0x0300,x`). This offsets the address by the unsigned value of x.

Indirect addressing is signified by placing the address or label in brackets (for example, `lda (0x0300)`). This gets the two bytes located at the given address, and uses those as the actual address for the instruction.

Indirect and offset addressing can be combined like so: `lda (0x0300),x`. This will get the address at the specified address, and then offset it.

`lda` also supports literal addressing, signified by placing a `#` in front of the operand (for example, `lda #255`). This will cause the processor to load specifically that value, without needing an address.

## ASSEMBLER DIRECTIVES AND LABELS

The AVC assembler supports 2 directives: `org` and `dat`. 

`org` positions the following instruction or directive at the specified position in the binary file. For example,  
```
org 0x0300
lda #200
```  
will position `lda #200` at the address 0x0300. All further instructions will follow on from this position.

`dat` places a byte (`dat 0x10`) or string literal (`dat "string"`) in the binary. Strings are encoded using ASCII and are not zero-terminated by default.

Labels can be created by placing `LABEL:` at the start of a line, where `LABEL` is the name of the label. These can then be later referenced by any instruction that uses an address, and can be offset and indirected as normal. When referenced, labels can be manipulated using simple arithmetic, evaluated at compile-time. A `+` or `-` may be applied, followed by an integer literal (see below). For example,  
```
sta DATA+1
```  
will store to the byte directly after the location of DATA.

### MACROS AND DECLARATIONS

At the start of a program file, declarations may be made. If the first line begins with `#`, all lines will be treated as declarations until the assembler reaches `#ENDD` on a line on its own. Currently, the assembler only supports one non-macro declaration: `#BYTE`. This will store a constant value in a dictionary, which can then be retrieved by any instruction that uses literal values (currently only `lda`). The syntax is `#BYTE NAME VALUE`.

Macros are more complicated. When the assembler sees a `#MACR` tag, it will add all following lines into a buffer, until it sees the an `#ENDM` tag. The syntax for the opening tag is `#MACR NAME`. A macro can later be invoked in the assembly with `!NAME`, where `NAME` is the name of the macro. This can also be followed by a list of arguments. Arguments are substituted into the text in a manner similar to shell scripts: `$1` refers to the first argument, `$2` refers to the second, etc.

## INTEGER LITERALS

Integer literals may be in base-2, 10, or 16, signified by `0b`, `0d`, and `0x` respectively. A literal without a specified radix is assumed to be in decimal.
