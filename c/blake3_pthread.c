#define _POSIX_C_SOURCE 200112L // for pthread barriers

#include <assert.h>
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <pthread.h>

#include "blake3_impl.h"

struct thread_context {
  uint32_t key[8];
  uint8_t flags;
  const uint8_t *l_input;
  size_t l_input_len;
  uint64_t l_chunk_counter;
  uint8_t *l_cvs;
  size_t *l_n;
};

struct blake3_thread {
  struct blake3_thread *next;
  pthread_t thread;
  pthread_barrier_t barrier;
  struct thread_context *ctx;
  bool exit;
};

static int n_threads = 0;
static struct blake3_thread *thread_list = NULL;
static pthread_mutex_t thread_list_mutex = PTHREAD_MUTEX_INITIALIZER;

static void *do_work(void *p) {
  struct blake3_thread *entry = p;

  for (;;) {
    pthread_barrier_wait(&entry->barrier);

    if (entry->exit)
      break;

    struct thread_context *ctx = entry->ctx;
    *ctx->l_n = blake3_compress_subtree_wide(ctx->l_input, ctx->l_input_len,
                                             ctx->key, ctx->l_chunk_counter,
                                             ctx->flags, ctx->l_cvs, true);

    pthread_barrier_wait(&entry->barrier);
  }

  return NULL;
}

static void thread_list_insert(struct blake3_thread *thread) {
  pthread_mutex_lock(&thread_list_mutex);
  thread->next = thread_list;
  thread_list = thread;
  pthread_mutex_unlock(&thread_list_mutex);
}

static int get_core_count(void)
{
#ifdef _SC_NPROCESSORS_ONLN
  static int n_cores;
  if (!n_cores)
    n_cores = sysconf(_SC_NPROCESSORS_ONLN);
  return n_cores;
#else
  return 4; /* Guess */
#endif
}

static struct blake3_thread *get_thread(void) {
  int max_thread_count = get_core_count();
  struct blake3_thread *thread = NULL;
  pthread_mutex_lock(&thread_list_mutex);

  if (thread_list) {
    thread = thread_list;
    thread_list = thread->next;
    thread->next = NULL;
    goto out;
  }

  if (n_threads < max_thread_count) {
    n_threads++;
    thread = calloc(1, sizeof(*thread));
    pthread_barrier_init(&thread->barrier, NULL, 2);
    pthread_create(&thread->thread, NULL, do_work, thread);
    goto out;
  }

out:
  pthread_mutex_unlock(&thread_list_mutex);
  return thread;
}

void blake3_compress_subtree_wide_join_pthread(
    // shared params
    const uint32_t key[8], uint8_t flags, bool use_threads,
    // left-hand side params
    const uint8_t *l_input, size_t l_input_len, uint64_t l_chunk_counter,
    uint8_t *l_cvs, size_t *l_n,
    // right-hand side params
    const uint8_t *r_input, size_t r_input_len, uint64_t r_chunk_counter,
    uint8_t *r_cvs, size_t *r_n) {

  struct blake3_thread *thread = use_threads ? get_thread() : NULL;

  if (!thread) {
    *l_n = blake3_compress_subtree_wide(l_input, l_input_len, key,
                                        l_chunk_counter, flags, l_cvs,
                                        use_threads);
    *r_n = blake3_compress_subtree_wide(r_input, r_input_len, key,
                                        r_chunk_counter, flags, r_cvs,
                                        use_threads);
    return;
  }

  struct thread_context ctx = {
    .flags = flags,
    .l_input = l_input,
    .l_input_len = l_input_len,
    .l_chunk_counter = l_chunk_counter,
    .l_cvs = l_cvs,
    .l_n = l_n,
  };
  memcpy(ctx.key, key, sizeof(ctx.key));

  thread->ctx = &ctx;
  pthread_barrier_wait(&thread->barrier);

  *r_n = blake3_compress_subtree_wide(r_input, r_input_len, key,
                                      r_chunk_counter, flags, r_cvs,
                                      use_threads);

  pthread_barrier_wait(&thread->barrier);
  thread->ctx = NULL;

  thread_list_insert(thread);
}

void blake3_pthread_reap(void)
{
  for (;;) {
    struct blake3_thread *thread = thread_list;
    if (!thread)
      break;

    thread_list = thread->next;
    thread->next = NULL;

    thread->exit = true;
    pthread_barrier_wait(&thread->barrier);
    pthread_join(thread->thread, NULL);
    pthread_barrier_destroy(&thread->barrier);
    free(thread);

    --n_threads;
  }

  assert(n_threads == 0);
}

__attribute__((destructor))
static void blake3_pthread_dtor(void) {
  blake3_pthread_reap();
}
