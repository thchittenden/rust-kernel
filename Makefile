# File config.
TARGET := kernel
SRCDIR := src
OBJDIR := obj
DEPDIR := dep
LIBDIR := lib
BINDIR := bin
IMGDIR := img
INCDIR := inc
DOCDIR := doc
USRDIR := usr
TARGETSPEC := target
LINKERSCRIPT := linker.ld

RAMDISK := $(USRDIR)/$(BINDIR)/ramdisk.bin
RAMDISK_DIR := $(USRDIR)/$(BINDIR)/ramdisk_dir.bin

# Build config.
LOG_LEVEL  := trace
LOG_DEVICE := serial

# Module config. This order is important (and fragile!)
CRATES := util mutex interrupt alloc collections sync io mem task sched fs devices rt boot

# Program config.
AS := gcc
ASFLAGS := -c -m32 -g
LD := ld
LDFLAGS := -melf_i386 -T $(LINKERSCRIPT) -static --gc-sections
CC := gcc
CCFLAGS := -m32 -c -ggdb -I$(INCDIR) 
RUSTC := rustc
RUSTCFLAGS := -O -L$(OBJDIR) -L$(LIBDIR) --target $(TARGETSPEC) -g --cfg 'LOG_DEVICE="$(LOG_DEVICE)"' --cfg 'LOG_LEVEL="$(LOG_LEVEL)"'
RUSTDOC := rustdoc
RUSTDOCFLAGS := -L$(OBJDIR) -L$(LIBDIR) --target $(TARGETSPEC)

# Utility functions.
reverse = $(if $(1),$(call reverse,$(wordlist 2,$(words $(1)),$(1)))) $(firstword $(1))
objectify = $(subst $(SRCDIR)/,$(OBJDIR)/,$(1:.$(2)=.$(3)))

# Find all source files and their corresponding object files.
RUST_SRCS := $(patsubst %,$(SRCDIR)/%/mod.rs,$(CRATES))
RUST_OBJS := $(patsubst %,$(OBJDIR)/lib%.rlib,$(CRATES))
ASM_SRCS := $(shell find $(SRCDIR) -name '*.S')
ASM_OBJS := $(call objectify,$(ASM_SRCS),S,o)
C_SRCS := $(shell find $(SRCDIR) -name '*.c')
C_OBJS := $(call objectify,$(C_SRCS),c,o)
OBJ_FILES := $(ASM_OBJS) $(C_OBJS) $(RUST_OBJS)
DEP_FILES := $(patsubst %,$(DEPDIR)/%.d,$(CRATES))

all: image

# Include all dependency files.
CUR_DEP_FILES := $(shell [ -d $(DEPDIR) ] && find $(DEPDIR) -name '*.d')
-include $(CUR_DEP_FILES)

# Build targets.
$(BINDIR)/$(TARGET): $(OBJ_FILES) $(RAMDISK) $(RAMDISK_DIR)
	@mkdir -p $(@D)
	$(LD) $(LDFLAGS) -o $@ --start-group $(call reverse,$^) $(LIBDIR)/libcore.rlib --end-group
	@-objdump -d $(BINDIR)/$(TARGET) | ./checkstack.py 2048

# The ramdisk is might by a subdirectory make file.
$(RAMDISK_DIR):
$(RAMDISK): 
	$(MAKE) -C $(USRDIR)

# We first check if compilation succeeds before we emit the dep-info because otherwise the 
# dependency file will be updated even if compilation fails and Make will try to build twice which
# is annoying.
$(DEPDIR)/%.d: $(SRCDIR)/%/mod.rs
	@mkdir -p $(@D)
	@-$(RUSTC) $(RUSTCFLAGS) -Z no-trans $< 2> /dev/null \
		&& $(RUSTC) $(RUSTCFLAGS) --emit dep-info -o $@ $< 2> /dev/null \
		&& ./getdeps.py $@ $< $(OBJDIR) $(CRATES) >> $@ \
		&& sed -i $@ -e s^$@^$(OBJDIR)/lib$(*F).rlib^g

$(OBJDIR)/lib%.rlib: $(DEPDIR)/%.d
	@mkdir -p $(@D)
	$(RUSTC) $(RUSTCFLAGS) -o $@ $(SRCDIR)/$(*F)/mod.rs

$(OBJDIR)/%.o: $(SRCDIR)/%.c
	@mkdir -p $(@D)
	$(CC) $(CCFLAGS) -o $@ $^

$(OBJDIR)/%.o: $(SRCDIR)/%.S
	@mkdir -p $(@D)
	$(AS) $(ASFLAGS) -o $@ $^

$(BINDIR)/$(TARGET).iso: $(BINDIR)/$(TARGET)
	cp $(BINDIR)/$(TARGET) $(IMGDIR)/boot/
	grub-mkrescue -o $(BINDIR)/$(TARGET).iso $(IMGDIR)

# Misc target.
image: $(BINDIR)/$(TARGET).iso

libcore:
	# Not portable really.
	rustc -o lib/libcore.rlib --target target -g -O ../rust/src/libcore/lib.rs

doc: $(addprefix doc-, $(CRATES))
doc-%: $(SRCDIR)/%/mod.rs $(DEPDIR)/%.d
	$(RUSTDOC) $(RUSTDOCFLAGS) -o $(DOCDIR) $<

clean: 
	rm -Rf $(BINDIR)
	rm -Rf $(OBJDIR)
	rm -Rf $(DEPDIR)
	rm -Rf $(DOCDIR)
	rm -f $(IMGDIR)/boot/$(TARGET)
	$(MAKE) -C $(USRDIR) clean

# Debug target.
print-%:
	@echo '$*=$($*)' 

.PHONY: all image clean doc libcore
.SECONDARY:

