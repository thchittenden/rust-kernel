# File config.
TARGET := kernel
SRCDIR := src
OBJDIR := obj
DEPDIR := dep
LIBDIR := lib
BINDIR := bin
IMGDIR := img
TARGETSPEC := target
LINKERSCRIPT := linker.ld

# Module config.
CRATES := sync console util rt mem boot

# Program config.
AS := gcc
ASFLAGS := -c -m32 -g
LD := ld
LDFLAGS := -melf_i386 -T $(LINKERSCRIPT) -static --gc-sections
CC := gcc
CCFLAGS := -g
RUSTC := rustc
RUSTCFLAGS := -O -L$(OBJDIR) -L$(LIBDIR) --target $(TARGETSPEC) -g

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
OBJ_FILES := $(RUST_OBJS) $(C_OBJS) $(ASM_OBJS)

all: image

# Include all dependency files.
DEP_FILES := $(shell [ -d $(DEPDIR) ] && find $(DEPDIR) -name '*.d')
-include $(DEP_FILES)

# Build targets.
$(BINDIR)/$(TARGET): $(OBJ_FILES)
	@mkdir -p $(@D)
	$(LD) $(LDFLAGS) -o $@ --start-group $(call reverse,$^) $(LIBDIR)/libcore.rlib --end-group

$(DEPDIR)/%.d: $(SRCDIR)/%/mod.rs
	@mkdir -p $(@D)
	-$(RUSTC) $(RUSTCFLAGS) --emit dep-info -o $@ $< 2> /dev/null

$(OBJDIR)/lib%.rlib: $(SRCDIR)/%/mod.rs $(DEPDIR)/%.d
	@mkdir -p $(@D)
	$(RUSTC) $(RUSTCFLAGS) -o $@ $< 

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

clean: 
	rm -Rf $(BINDIR)
	rm -Rf $(OBJDIR)
	rm -Rf $(DEPDIR)
	rm -f $(IMGDIR)/boot/$(TARGET)

# Debug target.
print-%:
	@echo '$*=$($*)' 

.PHONY: all image clean
.SECONDARY:

