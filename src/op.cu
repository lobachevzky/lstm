#include <stdio.h> 
#include <stdlib.h> 
#include <math.h> 
#include <cuda_runtime.h> 
#include "cublas_v2.h" 
#include "matrix.h" 

#define UN_MAP(name, f_body) \
  __device__ \
  float f_ ## name(float x) { \
    return f_body; \
  } \
  __global__ \
  void _ ## name(int len, float *result, float *a) { \
    SET(result, f_ ## name(a[IDx])) \
  } \
  void map_ ## name(Matrix *m, Matrix *result) { \
    DEFAULT_LAUNCH(_ ## name, result, m->devArray); \
  }

#define BIN_BROADCAST(name, op) \
  __global__ \
  void _ ## name ## _scalar(int len, float *result, float *a, float val) { \
    SET(result, val op a[IDx]) \
  } \
  void broadcast_ ## name(float val, Matrix *m, Matrix *result) { \
    DEFAULT_LAUNCH(_ ## name ## _scalar, result, m->devArray, val); \
  }

#define BIN_ELEMWISE(name, op) \
  __global__ \
  void _ ## name (int len, float *result, float *a1, float *a2) { \
    SET(result, a1[IDx] op a2[IDx]) \
  } \
  void elemwise_ ## name (Matrix *m1, Matrix *m2, Matrix *result) { \
    check_dims(m1, m2, result); \
    DEFAULT_LAUNCH(_ ## name, result, m1->devArray, m2->devArray); \
  }

void check_dims(Matrix *m1, Matrix *m2, Matrix *result) { 
  check(m1->height != m2->height 
     || m1->width  != m2->width
     || m1->height != result->height 
     || m1->width  != result->width, 
      "matrices must have the same dimensions");
}


UN_MAP(neg, -x) // map_neg
BIN_ELEMWISE(mult, *) // elemwise_mult
BIN_ELEMWISE(add, +) // elemwise_add
BIN_BROADCAST(mult, *) // broadcast_mult
BIN_BROADCAST(add, +) // broadcast_add
