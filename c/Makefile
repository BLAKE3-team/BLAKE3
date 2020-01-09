NAME=blake3
CC=gcc
CFLAGS=-O3 -Wall -Wextra -std=c11 -pedantic

all: blake3.c blake3_dispatch.c blake3_portable.c main.c blake3_sse41.o blake3_avx2.o blake3_avx512.o
	$(CC) $(CFLAGS) $^ -o $(NAME)

blake3_sse41.o: blake3_sse41.c
	$(CC) $(CFLAGS) -c $^ -o $@ -msse4.1 -D BLAKE3_USE_SSE41

blake3_avx2.o: blake3_avx2.c # blake3_sse41.c
	$(CC) $(CFLAGS) -c $^ -o $@ -mavx2 -D BLAKE3_USE_SSE41 -D BLAKE3_USE_AVX2

blake3_avx512.o: blake3_avx512.c
	$(CC) $(CFLAGS) -c $^ -o $@ -mavx512f -mavx512vl -D BLAKE3_USE_SSE41 -D BLAKE3_USE_AVX2 -D BLAKE3_USE_AVX512

blake3_neon.o: blake3_neon.c
	$(CC) $(CFLAGS) -c $^ -o $@ -D BLAKE3_USE_NEON

test: CFLAGS += -DBLAKE3_TESTING
test: all
	./test.py

clean: 
	rm -f $(NAME) *.o