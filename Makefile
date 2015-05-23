# File config.
TARGET := kernel
SRCDIR := src
OBJDIR := obj
BINDIR := bin
IMGDIR := img
TARGETSPEC := target

# Program config.
AS := as
ASFLAGS := --32
LD := ld
LDFLAGS := -melf_i386
CC := gcc
CCFLAGS := 
RUSTC := rustc

# Find all source files and their corresponding object files.
objectify = $(subst $(SRCDIR)/,$(OBJDIR)/,$(1:.$(2)=.o))
RUST_SRCS := $(shell find $(SRCDIR) -name '*.rs')
RUST_OBJS := $(call objectify,$(RUST_SRCS),rs)
ASM_SRCS := $(shell find $(SRCDIR) -name '*.S')
ASM_OBJS := $(call objectify,$(ASM_SRCS),S)
C_SRCS := $(shell find $(SRCDIR) -name '*.c')
C_OBJS := $(call objectify,$(C_SRCS),c)
OBJ_FILES := $(RUST_OBJS) $(ASM_OBJS) $(C_OBJS)

# Build targets.
$(BINDIR)/$(TARGET): $(OBJ_FILES)
	@mkdir -p $(@D)
	$(LD) $(LDFLAGS) -o $@ $^

$(OBJDIR)/%.o: $(SRCDIR)/%.rs
	@mkdir -p $(@D)
	$(RUSTC) --emit=obj --target $(TARGETSPEC) -o $@ $^ 

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

