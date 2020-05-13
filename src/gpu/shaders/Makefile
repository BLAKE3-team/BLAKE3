GLSLC=glslc

.PHONY: all
all: blake3-chunk-be.spv blake3-chunk-le.spv blake3-parent.spv

blake3-chunk-be.spv: blake3.comp
	$(GLSLC) -DCHUNK -DENDIAN -O -o $@ $<

blake3-chunk-le.spv: blake3.comp
	$(GLSLC) -DCHUNK -O -o $@ $<

blake3-parent.spv: blake3.comp
	$(GLSLC) -DPARENT -O -o $@ $<

.DELETE_ON_ERROR:
