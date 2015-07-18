# File config.
SRCDIR := src
OBJDIR := obj
BINDIR := bin
INCDIR := inc
RTDIR := rt
PROGSDIR := progs

# Program config.
AS := gcc
ASFLAGS := -m32 -c
CC := gcc
CCFLAGS := -m32 -I$(INCDIR) -static -nostdlib
LDFLAGS := -r -m elf_i386

# Discover all source files and infer output files.
RT_C_SRCS := $(shell find $(SRCDIR)/$(RTDIR) -name "*.c")
RT_S_SRCS := $(shell find $(SRCDIR)/$(RTDIR) -name "*.S")
RT_C_OBJS := $(patsubst $(SRCDIR)/%.c,$(OBJDIR)/%.o,$(RT_C_SRCS))
RT_S_OBJS := $(patsubst $(SRCDIR)/%.S,$(OBJDIR)/%.o,$(RT_S_SRCS))
RT_OBJS := $(RT_C_OBJS) $(RT_S_OBJS)
PROG_SRCS := $(shell find $(SRCDIR)/$(PROGSDIR) -name "*.c")
PROG_OBJS := $(patsubst $(SRCDIR)/%.c,$(OBJDIR)/%.o,$(PROG_SRCS))
PROG_BINS := $(patsubst $(OBJDIR)/%.o,$(OBJDIR)/%,$(PROG_OBJS))

RAMDISK_DIR_BIN := $(BINDIR)/ramdisk_dir.bin
RAMDISK_DIR := $(BINDIR)/ramdisk_dir
RAMDISK_FILES := $(patsubst %,%.bin.o,$(PROG_BINS))
RAMDISK := $(BINDIR)/ramdisk.bin

all: $(RAMDISK) $(RAMDISK_DIR_BIN)

$(RAMDISK): $(RAMDISK_FILES)
	@mkdir -p $(@D)
	$(LD) $(LDFLAGS) -o $@ $^

$(RAMDISK_DIR_BIN): $(RAMDISK_DIR)
	@mkdir -p $(@D)
	objcopy -I binary -O elf32-i386 -B i386 --rename-section .data=.ramdisk_dir $^ $@

$(RAMDISK_DIR): $(RAMDISK)
	@mkdir -p $(@D)
	readelf -W -a $^ | ./mkdir.py > $@

$(OBJDIR)/$(PROGSDIR)/%.bin.o: $(OBJDIR)/$(PROGSDIR)/%
	@mkdir -p $(@D)
	objcopy -I binary -O elf32-i386 -B i386 --rename-section .data=.ramdisk $^ $@

$(OBJDIR)/$(PROGSDIR)/%: $(OBJDIR)/$(PROGSDIR)/%.o $(RT_OBJS)
	@mkdir -p $(@D)
	$(CC) $(CCFLAGS) -Xlinker -T progs.ld -o $@ $^

$(RT_C_OBJS): $(OBJDIR)/$(RTDIR)/%.o: $(SRCDIR)/$(RTDIR)/%.c
	@mkdir -p $(@D)
	$(CC) $(CCFLAGS) -c -o $@ $^

$(RT_S_OBJS): $(OBJDIR)/$(RTDIR)/%.o: $(SRCDIR)/$(RTDIR)/%.S
	@mkdir -p $(@D)
	$(AS) $(ASFLAGS) -o $@ $^

$(PROG_OBJS): $(OBJDIR)/$(PROGSDIR)/%.o: $(SRCDIR)/$(PROGSDIR)/%.c
	@mkdir -p $(@D)
	$(CC) $(CCFLAGS) -c -o $@ $^

# Miscellaneous targets.
clean: 
	rm -Rf $(OBJDIR) $(BINDIR)

# Debug target.
print-%:
	@echo '$*=$($*)' 

.PHONY: all clean
.SECONDARY:
