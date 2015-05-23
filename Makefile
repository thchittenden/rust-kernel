# File config.
TARGET := kernel
SRCDIR := src
OBJDIR := obj
LIBDIR := lib
BINDIR := bin
IMGDIR := img
TARGETSPEC := target
LINKERSCRIPT := linker.ld

# Module config.
CRATES := boot rt

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
RUST_SRCS := $(patsubst %,$(SRCDIR)/%/lib.rs,$(CRATES))
RUST_OBJS := $(patsubst %,$(OBJDIR)/%.rlib,$(CRATES))
ASM_SRCS := $(shell find $(SRCDIR) -name '*.S')
ASM_OBJS := $(call objectify,$(ASM_SRCS),S,o)
C_SRCS := $(shell find $(SRCDIR) -name '*.c')
C_OBJS := $(call objectify,$(C_SRCS),c,o)
OBJ_FILES := $(ASM_OBJS) $(C_OBJS) $(RUST_OBJS)

# Build targets.
$(BINDIR)/$(TARGET): $(OBJ_FILES)
	@mkdir -p $(@D)
	$(LD) $(LDFLAGS) -o $@ $^

$(OBJDIR)/%.rlib: $(SRCDIR)/%/lib.rs
	@mkdir -p $(@D)
	$(RUSTC) $(RUSTCFLAGS) -o $@ $^ 

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
	rm -Rf $(BINDIR) $(OBJDIR)
	rm -f $(IMGDIR)/boot/$(TARGET)

# Debug target.
print-%:
	@echo '$*=$($*)'

