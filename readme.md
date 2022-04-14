### AVC: AJAL VIRTUAL CPU

avc_asm is a command line assembler and virtual machine for the AVC instruction set architecture. information about the processor, instruction set and assembly language is contained in `avc.md`.

## PROGRAM ARGUMENTS

`avc -{A|R} [OPTIONS] FILE` 

`-A` assembles a source file. `-R` runs an assembled binary file. `FILE` is the file to operate on.

Other options are:

`-o FILE`: specifies an output file for assembly. The default is `a.out`.

`-d DEBUG`: specifies a debug level. 0 is no debug information, 1 is a register readout on a break instruction, 2 is register readout on break and a readout of every instruction as it is executed, and 3 is a register readout every clock cycle.
