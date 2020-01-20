NAME=blake3
CC=gcc
CFLAGS=-O3 -Wall -Wextra -std=c11 -pedantic
BLAKE3_NO_NEON ?= 1
TARGETS=
EXTRAFLAGS=-DBLAKE3_TESTING

ifdef BLAKE3_NO_SSE41
EXTRAFLAGS += -DBLAKE3_NO_SSE41
else
TARGETS += blake3_sse41.o
endif

ifdef BLAKE3_NO_AVX2
EXTRAFLAGS += -DBLAKE3_NO_AVX2
else
TARGETS += blake3_avx2.o
endif

ifdef BLAKE3_NO_AVX512
EXTRAFLAGS += -DBLAKE3_NO_AVX512
else
TARGETS += blake3_avx512.o
endif

ifdef BLAKE3_NO_NEON
EXTRAFLAGS += -DBLAKE3_NO_NEON
else
TARGETS += blake3_neon.o
endif

all: blake3.c blake3_dispatch.c blake3_portable.c main.c $(TARGETS)
	$(CC) $(CFLAGS) $(EXTRAFLAGS) $^ -o $(NAME)

blake3_sse41.o: blake3_sse41.c
	$(CC) $(CFLAGS) $(EXTRAFLAGS) -c $^ -o $@ -msse4.1

blake3_avx2.o: blake3_avx2.c # blake3_sse41.c
	$(CC) $(CFLAGS) $(EXTRAFLAGS) -c $^ -o $@ -mavx2

blake3_avx512.o: blake3_avx512.c
	$(CC) $(CFLAGS) $(EXTRAFLAGS) -c $^ -o $@ -mavx512f -mavx512vl

blake3_neon.o: blake3_neon.c
	$(CC) $(CFLAGS) $(EXTRAFLAGS) -c $^ -o $@

test: CFLAGS += -DBLAKE3_TESTING
test: all
	./test.py

clean: 
	rm -f $(NAME) *.o
