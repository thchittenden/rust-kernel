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
CRATES := boot rt console macros

# Program config.
AS := gcc
ASFLAGS := -c -m32
LD := ld
LDFLAGS := -melf_i386 -T $(LINKERSCRIPT) -static
CC := gcc
CCFLAGS := 
RUSTC := rustc
RUSTCFLAGS := -L$(LIBDIR) -lcore --crate-type=rlib --target $(TARGETSPEC)

# Find all source files and their corresponding object files.
objectify = $(subst $(SRCDIR)/,$(OBJDIR)/,$(1:.$(2)=.$(3)))
RUST_SRCS := $(patsubst %,$(SRCDIR)/%/mod.rs,$(CRATES))
RUST_OBJS := $(patsubst %,$(OBJDIR)/%.rlib,$(CRATES))
ASM_SRCS := $(shell find $(SRCDIR) -name '*.S')
ASM_OBJS := $(call objectify,$(ASM_SRCS),S,o)
C_SRCS := $(shell find $(SRCDIR) -name '*.c')
C_OBJS := $(call objectify,$(C_SRCS),c,o)
OBJ_FILES := $(ASM_OBJS) $(C_OBJS) $(RUST_OBJS)

all: $(BINDIR)/$(TARGET)

# Include all dependency files.
DEP_FILES := $(shell find $(DEPDIR) -name '*.d')
-include $(DEP_FILES)

# Build targets.
$(BINDIR)/$(TARGET): $(OBJ_FILES)
	@mkdir -p $(@D)
	$(LD) $(LDFLAGS) -o $@ $^

$(DEPDIR)/%.d: $(SRCDIR)/%/mod.rs
	@mkdir -p $(@D)
	$(RUSTC) $(RUSTCFLAGS) --emit dep-info -o $@ $<

$(OBJDIR)/%.rlib: $(SRCDIR)/%/mod.rs $(DEPDIR)/%.d
	@mkdir -p $(@D)
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(OBJDIR)/%.o: $(SRCDIR)/%.c
	@mkdir -p $(@D)
	$(CC) $(CCFLAGS) -o $@ $^

$(OBJDIR)/%.o: $(SRCDIR)/%.S
	@mkdir -p $(@D)
	$(AS) $(ASFLAGS) -o $@ $^

# Misc target.
image: 
	cp $(BINDIR)/$(TARGET) $(IMGDIR)/boot/
	grub-mkrescue -o $(BINDIR)/$(TARGET).iso $(IMGDIR)

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

