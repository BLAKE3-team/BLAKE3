GLSLC=glslc

.PHONY: all
all: blake3-chunk.spv blake3-parent.spv endian.spv

blake3-chunk.spv: blake3.comp
	$(GLSLC) -DCHUNK -O -o $@ $<

blake3-parent.spv: blake3.comp
	$(GLSLC) -DPARENT -O -o $@ $<

endian.spv: endian.comp
	$(GLSLC) -O -o $@ $<

.DELETE_ON_ERROR:
