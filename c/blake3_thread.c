/*
 * blake3_thread.h - minimal thread pool implementation for BLAKE3
 *
 * Copyright (c) 2023 Pantelis Antoniou <pantelis.antoniou@konsulko.com>
 *
 * Released under the BLAKE3 License (CC0 1.0 or Apache License 2.0)
 */
#define _DEFAULT_SOURCE
#include <assert.h>
#include <stdbool.h>
#include <string.h>
#include <alloca.h>
#include <pthread.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>

#if defined(__linux__)
#include <sys/syscall.h>
#include <linux/futex.h>
#endif

#include "blake3_impl.h"
#include "blake3_thread.h"

#undef BIT64
#define BIT64(x) ((uint64_t)1 << (x))

#define B3WORK_SHUTDOWN ((const blake3_thread_work *)(void *)-1)

#if defined(__linux__) && !defined(BLAKE3_THREAD_PORTABLE)

/* linux pedal to the metal implementation */
static inline int futex(_Atomic(uint32_t) *uaddr, int futex_op, uint32_t val, const struct timespec *timeout, uint32_t *uaddr2, uint32_t val3)
{
	return syscall(SYS_futex, uaddr, futex_op, val, timeout, uaddr2, val3);
}

static inline int fwait(_Atomic(uint32_t) *futexp)
{
	long s;
	uint32_t one = 1;

	while (!atomic_compare_exchange_strong(futexp, &one, 0)) {
		s = futex(futexp, FUTEX_WAIT, 0, NULL, NULL, 0);
		if (s == -1 && errno != EAGAIN)
			return -1;
	}
	return 0;
}

static inline int fpost(_Atomic(uint32_t) *futexp)
{
	long s;
	uint32_t zero = 0;

	if (atomic_compare_exchange_strong(futexp, &zero, 1)) {
		s = futex(futexp, FUTEX_WAKE, 1, NULL, NULL, 0);
		if (s == -1)
			return -1;
	}
	return 0;
}

static inline void blake3_thread_init_sync(blake3_thread *t)
{
	/* nothing more needed for futexes */
	atomic_store(&t->submit, 0);
	atomic_store(&t->done, 0);
}

static inline const blake3_thread_work *blake3_worker_wait_for_work(blake3_thread *t)
{
	int ret;

	(void)ret;	/* for when NDEBUG is set */
	ret = fwait(&t->submit);
	assert(!ret);
	return t->work;
}

static inline void blake3_worker_signal_work_done(blake3_thread *t, const blake3_thread_work *work)
{
	const blake3_thread_work *exp_work;

	/* note that the work won't be replaced if it's a shutdown */
	exp_work = work;
	if (!atomic_compare_exchange_strong(&t->work, &exp_work, NULL)) {
		assert(exp_work == B3WORK_SHUTDOWN);
		return;
	}

	(void)fpost(&t->done);
}

int blake3_thread_submit_work(blake3_thread *t, const blake3_thread_work *work)
{
	const blake3_thread_work *exp_work;

	/* atomically update the work */
	exp_work = NULL;
	if (!atomic_compare_exchange_strong(&t->work, &exp_work, work)) {
		assert(exp_work == B3WORK_SHUTDOWN);
		return -1;
	}

	return fpost(&t->submit);
}

int blake3_thread_wait_work(blake3_thread *t)
{
	const blake3_thread_work *work;

	while ((work = atomic_load(&t->work)) != NULL)
		fwait(&t->done);

	atomic_store(&t->done, 0);

	return 0;
}

void blake3_worker_thread_shutdown(blake3_thread *t)
{
	atomic_store(&t->work, B3WORK_SHUTDOWN);
	fpost(&t->submit);
	pthread_join(t->tid, NULL);
}

#else

/* portable pthread implementation */

static inline void blake3_thread_init_sync(blake3_thread *t)
{
	pthread_mutex_init(&t->lock, NULL);
	pthread_cond_init(&t->cond, NULL);

	pthread_mutex_init(&t->wait_lock, NULL);
	pthread_cond_init(&t->wait_cond, NULL);
}

static inline const blake3_thread_work *blake3_worker_wait_for_work(blake3_thread *t)
{
	const blake3_thread_work *work;

	pthread_mutex_lock(&t->lock);
	while ((work = atomic_load(&t->work)) == NULL)
		pthread_cond_wait(&t->cond, &t->lock);
	pthread_mutex_unlock(&t->lock);

	return work;
}

static inline void blake3_worker_signal_work_done(blake3_thread *t, const blake3_thread_work *work)
{
	const blake3_thread_work *exp_work;

	/* clear the work, so that the user knows we're done */
	pthread_mutex_lock(&t->wait_lock);

	/* note that the work won't be replaced if it's a shutdown */
	exp_work = work;
	if (!atomic_compare_exchange_strong(&t->work, &exp_work, NULL))
		assert(exp_work == B3WORK_SHUTDOWN);
	pthread_cond_signal(&t->wait_cond);
	pthread_mutex_unlock(&t->wait_lock);
}

int blake3_thread_submit_work(blake3_thread *t, const blake3_thread_work *work)
{
	const blake3_thread_work *exp_work;
	int ret;

	/* atomically update the work */

	assert(t);
	assert(work);

	pthread_mutex_lock(&t->lock);
	exp_work = NULL;
	if (!atomic_compare_exchange_strong(&t->work, &exp_work, work)) {
		assert(exp_work == B3WORK_SHUTDOWN);
		ret = -1;
	} else {
		pthread_cond_signal(&t->cond);
		ret = 0;
	}
	pthread_mutex_unlock(&t->lock);

	return ret;
}

int blake3_thread_wait_work(blake3_thread *t)
{
	const blake3_thread_work *work;

	pthread_mutex_lock(&t->wait_lock);
	while ((work = atomic_load(&t->work)) != NULL)
		pthread_cond_wait(&t->wait_cond, &t->wait_lock);
	pthread_mutex_unlock(&t->wait_lock);

	return 0;
}

void blake3_worker_thread_shutdown(blake3_thread *t)
{
	pthread_mutex_lock(&t->lock);
	atomic_store(&t->work, B3WORK_SHUTDOWN);
	pthread_cond_signal(&t->cond);
	pthread_mutex_unlock(&t->lock);
	pthread_join(t->tid, NULL);
}

#endif

void *blake3_worker_thread(void *arg)
{
	blake3_thread *t = arg;
	const blake3_thread_work *work;

	while ((work = blake3_worker_wait_for_work(t)) != B3WORK_SHUTDOWN) {
		work->fn(work->arg);
		blake3_worker_signal_work_done(t, work);
	}

	return NULL;
}

blake3_thread *blake3_thread_pool_reserve(blake3_thread_pool *tp)
{
	blake3_thread *t;
	unsigned int slot;
	_Atomic(uint64_t) *free;
	uint64_t exp, v;
	unsigned int i;

	t = NULL;
	for (i = 0, free = tp->freep; i < tp->free_count; i++, free++) {
		v = atomic_load(free);
		while (v) {
			slot = highest_one(v);
			assert(v & BIT64(slot));
			exp = v;		/* expecting the previous value */
			v &= ~BIT64(slot);	/* clear this bit */
			if (atomic_compare_exchange_strong(free, &exp, v)) {
				slot += i * 64;
				t = tp->threads + slot;
				assert(slot == t->id);
				return t;
			}
			v = exp;
		}
	}

	return NULL;
}

void blake3_thread_pool_unreserve(blake3_thread_pool *tp, blake3_thread *t)
{
	_Atomic(uint64_t) *free;

	free = tp->freep + (unsigned int)(t->id / 64);
	atomic_fetch_or(free, BIT64(t->id & 63));
}

void blake3_thread_pool_cleanup(blake3_thread_pool *tp)
{
	blake3_thread *t;
	unsigned int i;

	if (!tp)
		return;

	if (tp->threads) {
		for (i = 0; i < tp->num_threads; i++) {
			t = &tp->threads[i];
			if (t->id == i)
				blake3_worker_thread_shutdown(t);
		}

		free(tp->threads);
	}

	if (tp->freep)
		free(tp->freep);

	memset(tp, 0, sizeof(*tp));
}

int blake3_thread_pool_init(blake3_thread_pool *tp, unsigned int num_threads)
{
	blake3_thread *t;
	unsigned int i;
	int rc;

	assert(tp);

	if (!num_threads) {
		long scval;
		scval = sysconf(_SC_NPROCESSORS_ONLN);
		assert(scval > 0);
		/* we spin num_cpus * 3 / 2 threads to cover I/O bubbles */
		num_threads = (unsigned int)((scval * 3) / 2);
	}

	memset(tp, 0, sizeof(*tp));

	tp->num_threads = num_threads;

	tp->free_count = (tp->num_threads / 64) + ((tp->num_threads & 63) ? 1 : 0);

	tp->freep = malloc(tp->free_count * sizeof(uint64_t));
	if (!tp->freep)
		goto err_out;

	for (i = 0; i < tp->free_count; i++)
		tp->freep[i] = (uint64_t)-1;
	if (tp->num_threads & 63)
		tp->freep[tp->free_count - 1] = BIT64(tp->num_threads & 63) - 1;

	tp->threads = malloc(sizeof(*tp->threads) * tp->num_threads);
	if (!tp->threads)
		goto err_out;

	memset(tp->threads, 0, sizeof(*tp->threads) * tp->num_threads);

	for (i = 0, t = tp->threads; i < tp->num_threads; i++, t++) {

		t->tp = tp;
		t->id = (unsigned int)-1;

		blake3_thread_init_sync(t);
	}

	for (i = 0, t = tp->threads; i < tp->num_threads; i++, t++) {

		rc = pthread_create(&t->tid, NULL, blake3_worker_thread, t);
		if (rc)
			goto err_out;
		t->id = i;
	}

	return 0;

err_out:
	blake3_thread_pool_cleanup(tp);
	return -1;
}

blake3_thread_pool *blake3_thread_pool_create(unsigned int num_threads)
{
	blake3_thread_pool *tp;
	int rc;

	tp = malloc(sizeof(*tp));
	if (!tp)
		return NULL;

	rc = blake3_thread_pool_init(tp, num_threads);
	if (rc) {
		free(tp);
		return NULL;
	}

	return tp;
}

void blake3_thread_pool_destroy(blake3_thread_pool *tp)
{
	if (!tp)
		return;

	blake3_thread_pool_cleanup(tp);
	free(tp);
}

void blake3_thread_work_join(blake3_thread_pool *tp, const blake3_thread_work *works, size_t work_count, blake3_work_check_fn check_fn)
{
	const blake3_thread_work **direct_work, **thread_work, *w;
	blake3_thread **threads, *t;
	size_t i, direct_work_count, thread_work_count;
	int rc;

	/* just a single (or no) work, or no threads? execute directly */
	if (work_count <= 1 || !tp || !tp->num_threads) {
		for (i = 0, w = works; i < work_count; i++, w++)
			w->fn(w->arg);
		return;
	}

	/* allocate the keeper of direct work */
	direct_work = alloca(work_count * sizeof(*direct_work));
	direct_work_count = 0;

	threads = alloca(work_count * sizeof(*threads));
	thread_work = alloca(work_count * sizeof(*thread_work));
	thread_work_count = 0;

	for (i = 0, w = works; i < work_count; i++, w++) {

		t = NULL;
		if (!check_fn || check_fn(w->arg))
			t = blake3_thread_pool_reserve(tp);

		if (t) {
			threads[thread_work_count] = t;
			thread_work[thread_work_count++] = w;
		} else
			direct_work[direct_work_count++] = w;
	}

	/* if we don't have any direct_work, steal the last threaded work as direct */
	if (!direct_work_count) {
		assert(thread_work_count > 0);
		t = threads[thread_work_count - 1];
		w = thread_work[thread_work_count - 1];
		thread_work_count--;

		/* unreserve this */
		blake3_thread_pool_unreserve(tp, t);
		direct_work[direct_work_count++] = w;
	}

	/* submit the threaded work */
	for (i = 0; i < thread_work_count; i++) {
		t = threads[i];
		w = thread_work[i];
		rc = blake3_thread_submit_work(t, w);
		if (rc) {
			/* unable to submit? remove work, and move to direct */
			threads[i] = NULL;
			thread_work[i] = NULL;
			blake3_thread_pool_unreserve(tp, t);
			direct_work[direct_work_count++] = w;
		}
	}

	/* now perform the direct work while the threaded work is being performed in parallel */
	for (i = 0; i < direct_work_count; i++) {
		w = direct_work[i];
		w->fn(w->arg);
	}

	/* finally wait for all threaded work to complete */
	for (i = 0; i < thread_work_count; i++) {
		t = threads[i];
		assert(t);
		blake3_thread_wait_work(t);
		blake3_thread_pool_unreserve(tp, t);
	}
}

void blake3_thread_args_join(blake3_thread_pool *tp, blake3_work_exec_fn fn, blake3_work_check_fn check_fn, void **args, size_t count)
{
	blake3_thread_work *works;
	size_t i;

	if (!count)
		return;

	works = alloca(sizeof(*works) * count);
	for (i = 0; i < count; i++) {
		works[i].fn = fn;
		works[i].arg = args ? args[i] : NULL;
	}

	blake3_thread_work_join(tp, works, count, check_fn);
}

void blake3_thread_arg_array_join(blake3_thread_pool *tp, blake3_work_exec_fn fn, blake3_work_check_fn check_fn, void *args, size_t argsize, size_t count)
{
	blake3_thread_work *works;
	uint8_t *p;
	size_t i;

	if (!count)
		return;

	works = alloca(sizeof(*works) * count);
	for (i = 0, p = args; i < count; i++, p += argsize) {
		works[i].fn = fn;
		works[i].arg = p;
	}

	blake3_thread_work_join(tp, works, count, check_fn);
}

void blake3_thread_arg_join(blake3_thread_pool *tp, blake3_work_exec_fn fn, blake3_work_check_fn check_fn, void *arg, size_t count)
{
	blake3_thread_work *works;
	size_t i;

	if (!count)
		return;

	works = alloca(sizeof(*works) * count);
	for (i = 0; i < count; i++) {
		works[i].fn = fn;
		works[i].arg = arg;
	}

	blake3_thread_work_join(tp, works, count, check_fn);
}
