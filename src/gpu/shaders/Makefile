GLSLC=glslc

.PHONY: all
all: blake3.spv endian.spv

blake3.spv: blake3.comp
	$(GLSLC) -O -o $@ $<

endian.spv: endian.comp
	$(GLSLC) -O -o $@ $<

.DELETE_ON_ERROR:
