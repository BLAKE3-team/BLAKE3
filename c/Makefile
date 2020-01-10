NAME=blake3
CC=gcc
CFLAGS=-O3 -Wall -Wextra -std=c11 -pedantic
BLAKE3_USE_SSE41 ?= 1
BLAKE3_USE_AVX2 ?= 1
BLAKE3_USE_AVX512 ?= 1
BLAKE3_USE_NEON ?= 0
TARGETS=
EXTRAFLAGS=-DBLAKE3_TESTING

ifeq ($(BLAKE3_USE_SSE41), 1)
TARGETS += blake3_sse41.o
EXTRAFLAGS += -DBLAKE3_USE_SSE41
endif

ifeq ($(BLAKE3_USE_AVX2), 1)
TARGETS += blake3_avx2.o
EXTRAFLAGS += -DBLAKE3_USE_AVX2
endif

ifeq ($(BLAKE3_USE_AVX512), 1)
TARGETS += blake3_avx512.o
EXTRAFLAGS += -DBLAKE3_USE_AVX512
endif

ifeq ($(BLAKE3_USE_NEON), 1)
TARGETS += blake3_neon.o
EXTRAFLAGS += -DBLAKE3_USE_NEON
endif

all: blake3.c blake3_dispatch.c blake3_portable.c main.c $(TARGETS)
	$(CC) $(CFLAGS) $(EXTRAFLAGS) $^ -o $(NAME)

blake3_sse41.o: blake3_sse41.c
	$(CC) $(CFLAGS) -c $^ -o $@ -msse4.1

blake3_avx2.o: blake3_avx2.c # blake3_sse41.c
	$(CC) $(CFLAGS) -c $^ -o $@ -mavx2

blake3_avx512.o: blake3_avx512.c
	$(CC) $(CFLAGS) -c $^ -o $@ -mavx512f -mavx512vl

blake3_neon.o: blake3_neon.c
	$(CC) $(CFLAGS) -c $^ -o $@ 

test: CFLAGS += -DBLAKE3_TESTING
test: all
	./test.py

clean: 
	rm -f $(NAME) *.o