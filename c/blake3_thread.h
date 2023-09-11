/*
 * blake3_thread.h - minimal thread pool implementation for BLAKE3
 *
 * Copyright (c) 2023 Pantelis Antoniou <pantelis.antoniou@konsulko.com>
 *
 * Released under the BLAKE3 License (CC0 1.0 or Apache License 2.0)
 */
#ifndef BLAKE3_THREAD_H
#define BLAKE3_THREAD_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdatomic.h>
#include <pthread.h>

struct blake3_thread_pool;

typedef void (*blake3_work_exec_fn)(void *arg);
typedef bool (*blake3_work_check_fn)(const void *arg);

typedef struct blake3_thread_work {
	blake3_work_exec_fn fn;
	void *arg;
} blake3_thread_work;

//#define BLAKE3_THREAD_PORTABLE
typedef struct blake3_thread {
	struct blake3_thread_pool *tp;
	unsigned int id;
	pthread_t tid;
	void *arg;
	_Atomic(const blake3_thread_work *)work;
#if defined(__linux__) && !defined(BLAKE3_THREAD_PORTABLE)
	_Atomic(uint32_t) submit;
	_Atomic(uint32_t) done;
#else
	pthread_mutex_t lock;
	pthread_cond_t cond;
	pthread_mutex_t wait_lock;
	pthread_cond_t wait_cond;
#endif
} blake3_thread;

typedef struct blake3_thread_pool {
	unsigned int num_threads;
	struct blake3_thread *threads;
	_Atomic(uint64_t) *freep;
	unsigned int free_count;
} blake3_thread_pool;

blake3_thread_pool *blake3_thread_pool_create(unsigned int num_threads);
void blake3_thread_pool_destroy(blake3_thread_pool *tp);
blake3_thread *blake3_thread_pool_reserve(blake3_thread_pool *tp);
void blake3_thread_pool_unreserve(blake3_thread_pool *tp, blake3_thread *t);
int blake3_thread_submit_work(blake3_thread *t, const blake3_thread_work *work);
int blake3_thread_wait_work(blake3_thread *t);

void blake3_thread_work_join(blake3_thread_pool *tp, const blake3_thread_work *works, size_t work_count, blake3_work_check_fn check_fn);
void blake3_thread_args_join(blake3_thread_pool *tp, blake3_work_exec_fn, blake3_work_check_fn check_fn, void **args, size_t count);
void blake3_thread_arg_array_join(blake3_thread_pool *tp, blake3_work_exec_fn fn, blake3_work_check_fn check_fn, void *args, size_t argsize, size_t count);
void blake3_thread_arg_join(blake3_thread_pool *tp, blake3_work_exec_fn fn, blake3_work_check_fn check_fn, void *arg, size_t count);

#endif
