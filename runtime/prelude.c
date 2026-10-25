/* ── sigi C Runtime Prelude ───────────────────────────────────────
 *
 * Stack-based runtime for compiled sigi programs.  Included verbatim
 * at the top of every generated C file.
 *
 * Stack: fixed-size array of typed values with overflow/underflow guards.
 * Arrays: dynamically allocated, tracked by ID; each element is a typed value.
 * Variables: 100 indexed slots for numbered vars.
 */

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>
#include <unistd.h>

#define STACK_SIZE 1000
#define MAX_ARRAYS 10

/* ── Typed Values ──────────────────────────────────────────────────
 * Each stack slot and variable carries a numeric type.  Untyped numbers
 * (`!5`, `!2.5`) are SG_F64.  Integers are stored sign-extended in `s`
 * (signed types) or zero-extended in `u` (unsigned types).  SG_F64 and
 * SG_F32 keep their value in `d`, already rounded to float for SG_F32.
 * SG_LD keeps its value in `ld`.  Conversions and the usual arithmetic
 * conversions follow C.
 */

typedef enum { SG_F64, SG_F32, SG_LD, SG_I8, SG_U8, SG_I16, SG_U16, SG_I32, SG_U32, SG_I64, SG_U64, SG_PTR } sg_ty;
typedef enum { SG_OP_ADD, SG_OP_SUB, SG_OP_MUL, SG_OP_DIV, SG_OP_MOD } sg_op;
typedef enum { SG_REL_EQ, SG_REL_LT, SG_REL_GT } sg_relop;

/* What a pointer addresses: a variable cell (an sv), one scalar member of
 * a struct, or a whole struct object. */
typedef enum { SG_PK_CELL, SG_PK_SCALAR, SG_PK_AGG, SG_PK_ARRAY } sg_pk;

typedef struct sg_agg sg_agg;

/* One member of a struct descriptor: its name, byte offset and kind. */
typedef struct {
    const char *name;
    size_t off;
    sg_pk kind;
    sg_ty pt;
    const sg_agg *agg;
} sg_field;

struct sg_agg {
    const char *name;
    size_t size;
    int nfields;
    const sg_field *fields;
};

typedef struct {
    void *addr;
    sg_pk kind;
    sg_ty pt;             /* scalar type when kind is SG_PK_SCALAR */
    const sg_agg *agg;    /* struct descriptor when kind is SG_PK_AGG */
    sg_pk ekind;          /* leaf kind of an SG_PK_ARRAY: SCALAR or AGG */
    const size_t *dims;   /* SG_PK_ARRAY: the extents of the array addressed */
    int nd;               /* number of entries in dims */
} sg_ptr;

typedef struct {
    sg_ty t;
    union { double d; long double ld; long long s; unsigned long long u; sg_ptr ptr; } v;
} sv;

static void sg_fail(const char *msg) { fprintf(stderr, "%s\n", msg); exit(1); }

static int sg_is_float(sg_ty t) { return t == SG_F64 || t == SG_F32 || t == SG_LD; }

static int sg_is_signed(sg_ty t) { return t == SG_I8 || t == SG_I16 || t == SG_I32 || t == SG_I64; }

static int sg_width(sg_ty t) {
    switch (t) {
    case SG_I8: case SG_U8: return 8;
    case SG_I16: case SG_U16: return 16;
    case SG_I32: case SG_U32: return 32;
    default: return 64;
    }
}

static unsigned long long sg_size(sg_ty t) {
    switch (t) {
    case SG_LD: return sizeof(long double);
    case SG_F32: return 4;
    case SG_I8: case SG_U8: return 1;
    case SG_I16: case SG_U16: return 2;
    case SG_I32: case SG_U32: return 4;
    default: return 8;
    }
}

/* Build an integer of type t from its two's-complement bits, wrapping to its width. */
static sv sg_from_bits(sg_ty t, unsigned long long bits) {
    int n = sg_width(t);
    unsigned long long mask = n == 64 ? ~0ULL : ((1ULL << n) - 1);
    sv r;
    r.t = t;
    bits &= mask;
    if (sg_is_signed(t)) {
        if (bits >> (n - 1)) r.v.s = -(long long)((~bits) & mask) - 1;
        else r.v.s = (long long)bits;
    } else {
        r.v.u = bits;
    }
    return r;
}

/* Two's-complement bits of an integer value. */
static unsigned long long sg_bits_of(sv x) {
    return sg_is_signed(x.t) ? (unsigned long long)x.v.s : x.v.u;
}

/* The value of a float-typed sv, widened to long double. */
static long double sg_fval(sv x) { return x.t == SG_LD ? x.v.ld : (long double)x.v.d; }

/* Convert a value to type `to`.  Integer-to-integer conversions wrap; a
 * float outside the target range is an error (C leaves it undefined). */
static sv sg_cast(sv x, sg_ty to) {
    sv r;
    if (x.t == SG_PTR || to == SG_PTR) sg_fail("Invalid conversion of a pointer");
    r.t = to;
    if (to == SG_F64) {
        r.v.d = sg_is_float(x.t) ? (double)sg_fval(x) : sg_is_signed(x.t) ? (double)x.v.s : (double)x.v.u;
        return r;
    }
    if (to == SG_F32) {
        if (sg_is_float(x.t)) r.v.d = (float)sg_fval(x);
        else if (sg_is_signed(x.t)) r.v.d = (float)x.v.s;
        else r.v.d = (float)x.v.u;
        return r;
    }
    if (to == SG_LD) {
        if (sg_is_float(x.t)) r.v.ld = sg_fval(x);
        else if (sg_is_signed(x.t)) r.v.ld = (long double)x.v.s;
        else r.v.ld = (long double)x.v.u;
        return r;
    }
    if (sg_is_float(x.t)) {
        long double t = truncl(sg_fval(x));
        int n = sg_width(to);
        if (sg_is_signed(to)) {
            long double lim = ldexpl(1.0L, n - 1);
            if (!(t >= -lim && t < lim)) sg_fail("Conversion out of range");
            return sg_from_bits(to, (unsigned long long)(long long)t);
        }
        if (!(t >= 0.0L && t < ldexpl(1.0L, n))) sg_fail("Conversion out of range");
        return sg_from_bits(to, (unsigned long long)t);
    }
    return sg_from_bits(to, sg_bits_of(x));
}

static double sg_to_double(sv x) { return sg_cast(x, SG_F64).v.d; }

static long long sg_as_ll(sv x) {
    if (x.t == SG_PTR) sg_fail("Pointer used as an integer");
    return sg_is_float(x.t) ? (long long)sg_fval(x) : (long long)sg_bits_of(x);
}

static int sg_truthy(sv x) {
    if (x.t == SG_PTR) return x.v.ptr.addr != NULL;
    return sg_is_float(x.t) ? sg_fval(x) != 0.0L : sg_bits_of(x) != 0; }

static sg_ty sg_promote(sg_ty t) {
    return (t == SG_I8 || t == SG_U8 || t == SG_I16 || t == SG_U16) ? SG_I32 : t;
}

/* The common type of a binary operation (C's usual arithmetic conversions). */
static sg_ty sg_common(sg_ty a, sg_ty b) {
    if (a == SG_LD || b == SG_LD) return SG_LD;
    if (a == SG_F64 || b == SG_F64) return SG_F64;
    if (a == SG_F32 || b == SG_F32) return SG_F32;
    a = sg_promote(a);
    b = sg_promote(b);
    if (a == SG_U64 || b == SG_U64) return SG_U64;
    if (a == SG_I64 || b == SG_I64) return SG_I64;
    if (a == SG_U32 || b == SG_U32) return SG_U32;
    return SG_I32;
}

/* Arithmetic in the common type.  Integer overflow wraps (C leaves it
 * undefined); integer division by zero is an error. */
static sv sg_ptr_arith(sg_op op, sv a, sv b);

static sv sg_arith(sg_op op, sv a, sv b) {
    if (a.t == SG_PTR || b.t == SG_PTR) return sg_ptr_arith(op, a, b);
    sg_ty t = sg_common(a.t, b.t);
    sv x = sg_cast(a, t), y = sg_cast(b, t), r;
    r.t = t;
    if (t == SG_LD) {
        long double p = x.v.ld, q = y.v.ld, out = 0.0L;
        switch (op) {
        case SG_OP_ADD: out = p + q; break;
        case SG_OP_SUB: out = p - q; break;
        case SG_OP_MUL: out = p * q; break;
        case SG_OP_DIV: out = p / q; break;
        case SG_OP_MOD: out = fmodl(p, q); break;
        }
        r.v.ld = out;
        return r;
    }
    if (t == SG_F64) {
        double p = x.v.d, q = y.v.d;
        switch (op) {
        case SG_OP_ADD: r.v.d = p + q; break;
        case SG_OP_SUB: r.v.d = p - q; break;
        case SG_OP_MUL: r.v.d = p * q; break;
        case SG_OP_DIV: r.v.d = p / q; break;
        case SG_OP_MOD: r.v.d = fmod(p, q); break;
        }
        return r;
    }
    if (t == SG_F32) {
        float p = (float)x.v.d, q = (float)y.v.d, out = 0.0f;
        switch (op) {
        case SG_OP_ADD: out = p + q; break;
        case SG_OP_SUB: out = p - q; break;
        case SG_OP_MUL: out = p * q; break;
        case SG_OP_DIV: out = p / q; break;
        case SG_OP_MOD: out = fmodf(p, q); break;
        }
        r.v.d = (double)out;
        return r;
    }
    unsigned long long p = sg_bits_of(x), q = sg_bits_of(y);
    switch (op) {
    case SG_OP_ADD: return sg_from_bits(t, p + q);
    case SG_OP_SUB: return sg_from_bits(t, p - q);
    case SG_OP_MUL: return sg_from_bits(t, p * q);
    default: break;
    }
    if (q == 0) sg_fail("Division by zero");
    if (sg_is_signed(t)) {
        long long sp = x.v.s, sq = y.v.s;
        if (sq == -1) return sg_from_bits(t, op == SG_OP_DIV ? 0ULL - p : 0ULL);
        return sg_from_bits(t, (unsigned long long)(op == SG_OP_DIV ? sp / sq : sp % sq));
    }
    return sg_from_bits(t, op == SG_OP_DIV ? p / q : p % q);
}

/* Relational test in the common type.  Unordered floats (NaN) compare false. */
static int sg_rel(sg_relop op, sv a, sv b) {
    if (a.t == SG_PTR || b.t == SG_PTR) {
        if (a.t != b.t) sg_fail("Comparison of a pointer with a number");
        uintptr_t p = (uintptr_t)a.v.ptr.addr, q = (uintptr_t)b.v.ptr.addr;
        switch (op) {
        case SG_REL_EQ: return p == q;
        case SG_REL_LT: return p < q;
        default: return p > q;
        }
    }
    sg_ty t = sg_common(a.t, b.t);
    sv x = sg_cast(a, t), y = sg_cast(b, t);
    int cmp;
    if (sg_is_float(t)) {
        long double p = sg_fval(x), q = sg_fval(y);
        if (p == q) cmp = 0;
        else if (p < q) cmp = -1;
        else if (p > q) cmp = 1;
        else return 0;
    } else if (sg_is_signed(t)) {
        cmp = (x.v.s > y.v.s) - (x.v.s < y.v.s);
    } else {
        cmp = (x.v.u > y.v.u) - (x.v.u < y.v.u);
    }
    switch (op) {
    case SG_REL_EQ: return cmp == 0;
    case SG_REL_LT: return cmp < 0;
    default: return cmp > 0;
    }
}

static void sg_print(sv x) {
    if (x.t == SG_PTR) printf("%p\n", x.v.ptr.addr);
    else if (x.t == SG_LD) printf("%Lg\n", x.v.ld);
    else if (sg_is_float(x.t)) printf("%g\n", x.v.d);
    else if (sg_is_signed(x.t)) printf("%lld\n", x.v.s);
    else printf("%llu\n", x.v.u);
}

static sv stack[STACK_SIZE];
static int sp = 0;
static sv vars[100];
static sv *arrays[MAX_ARRAYS];
static int array_sizes[MAX_ARRAYS];

/* ── Array Helpers ─────────────────────────────────────────────── */

static void arr_init(int id, int size) {
    if (id < 0 || id >= MAX_ARRAYS) { fprintf(stderr, "Array ID out of range\n"); exit(1); }
    arrays[id] = (sv *)calloc(size, sizeof(sv));
    array_sizes[id] = size;
}

static void arr_free(int id) {
    if (id < 0 || id >= MAX_ARRAYS) { fprintf(stderr, "Array ID out of range\n"); exit(1); }
    if (arrays[id]) { free(arrays[id]); arrays[id] = NULL; array_sizes[id] = 0; }
}

static sv *arr_slot(int id, int index) {
    if (id < 0 || id >= MAX_ARRAYS || !arrays[id]) { fprintf(stderr, "Array not allocated\n"); exit(1); }
    if (index < 0 || index >= array_sizes[id]) { fprintf(stderr, "Array index out of range\n"); exit(1); }
    return &arrays[id][index];
}

/* ── Integer Helpers ───────────────────────────────────────────── */

static int shift_count(double n) {
    if (n < 0.0 || n > 63.0) { fprintf(stderr, "Shift count out of range\n"); exit(1); }
    return (int)n;
}

/* ── Stack Operations ──────────────────────────────────────────── */

static void pushv(sv x) {
    if (sp >= STACK_SIZE) { fprintf(stderr, "Stack overflow\n"); exit(1); }
    stack[sp++] = x;
}

/* Push an untyped number. */
static void push(double x) {
    sv v;
    v.t = SG_F64;
    v.v.d = x;
    pushv(v);
}

static sv popv(void) {
    if (sp <= 0) { fprintf(stderr, "Stack underflow\n"); exit(1); }
    return stack[--sp];
}

/* Pop as a double, for operations that work on doubles. */
static double pop(void) {
    return sg_to_double(popv());
}

/* ── Pointers ──────────────────────────────────────────────────────
 * A pointer is an SG_PTR value.  It addresses a variable cell (an sv, from
 * `j.name` or `j<n>`), one scalar member of a struct (typed memory), or a
 * whole struct object.  `j` takes an address, `y` loads through one, `:`
 * stores through one, and `n.field` yields a pointer to a member. */

static sv sg_ptr_value(void *addr, sg_pk kind, sg_ty pt, const sg_agg *agg) {
    sv r;
    r.t = SG_PTR;
    r.v.ptr.addr = addr;
    r.v.ptr.kind = kind;
    r.v.ptr.pt = pt;
    r.v.ptr.agg = agg;
    r.v.ptr.ekind = kind;
    r.v.ptr.dims = NULL;
    r.v.ptr.nd = 0;
    return r;
}

static sv sg_ptr_to(sv *cell) { return sg_ptr_value(cell, SG_PK_CELL, SG_F64, NULL); }

static sv sg_agg_ptr(void *obj, const sg_agg *agg) {
    return sg_ptr_value(obj, SG_PK_AGG, SG_F64, agg);
}

/* A pointer to a multi-dimensional array with extents dims[0..nd): it
 * addresses the first of its elements, whose leaves are ekind/pt/agg. */
static sv sg_array_ptr(void *addr, sg_pk ekind, sg_ty pt, const sg_agg *agg, const size_t *dims, int nd) {
    sv r = sg_ptr_value(addr, SG_PK_ARRAY, pt, agg);
    r.v.ptr.ekind = ekind;
    r.v.ptr.dims = dims;
    r.v.ptr.nd = nd;
    return r;
}

static size_t sg_ty_size(sg_ty t) {
    switch (t) {
    case SG_F64: return sizeof(double);
    case SG_F32: return sizeof(float);
    case SG_LD: return sizeof(long double);
    case SG_I8: case SG_U8: return 1;
    case SG_I16: case SG_U16: return 2;
    case SG_I32: case SG_U32: return 4;
    default: return 8;
    }
}

/* Bytes spanned by one step of pointer arithmetic: the size of the object
 * the pointer addresses (for an array, the whole array). */
static size_t sg_step(sv p) {
    const sg_ptr *q = &p.v.ptr;
    switch (q->kind) {
    case SG_PK_SCALAR: return sg_ty_size(q->pt);
    case SG_PK_AGG: return q->agg->size;
    case SG_PK_ARRAY: {
        size_t n = q->ekind == SG_PK_AGG ? q->agg->size : sg_ty_size(q->pt);
        for (int i = 0; i < q->nd; i++) n *= q->dims[i];
        return n;
    }
    default: sg_fail("Pointer arithmetic needs a pointer to typed memory");
    }
    return 0;
}

/* Pointer plus or minus an integer, scaled by the size of the pointee. */
static sv sg_ptr_arith(sg_op op, sv a, sv b) {
    if (op != SG_OP_ADD && op != SG_OP_SUB) sg_fail("Invalid arithmetic on a pointer");
    if (a.t != SG_PTR) {
        if (b.t != SG_PTR || op == SG_OP_SUB) sg_fail("Pointer arithmetic needs a pointer operand");
        sv t = a; a = b; b = t;
    }
    if (b.t == SG_PTR) sg_fail("Pointer arithmetic needs an integer operand");
    long long n = sg_as_ll(b);
    if (op == SG_OP_SUB) n = -n;
    sv r = a;
    r.v.ptr.addr = (char *)a.v.ptr.addr + n * (long long)sg_step(a);
    return r;
}

/* Load a scalar of type t from typed memory. */
static sv sg_read_scalar(const void *addr, sg_ty t) {
    sv r;
    switch (t) {
    case SG_F64: { double x; memcpy(&x, addr, sizeof x); r.t = SG_F64; r.v.d = x; return r; }
    case SG_F32: { float x; memcpy(&x, addr, sizeof x); r.t = SG_F32; r.v.d = x; return r; }
    case SG_LD: { long double x; memcpy(&x, addr, sizeof x); r.t = SG_LD; r.v.ld = x; return r; }
    case SG_I8: { int8_t x; memcpy(&x, addr, sizeof x); return sg_from_bits(t, (unsigned long long)(long long)x); }
    case SG_U8: { uint8_t x; memcpy(&x, addr, sizeof x); return sg_from_bits(t, x); }
    case SG_I16: { int16_t x; memcpy(&x, addr, sizeof x); return sg_from_bits(t, (unsigned long long)(long long)x); }
    case SG_U16: { uint16_t x; memcpy(&x, addr, sizeof x); return sg_from_bits(t, x); }
    case SG_I32: { int32_t x; memcpy(&x, addr, sizeof x); return sg_from_bits(t, (unsigned long long)(long long)x); }
    case SG_U32: { uint32_t x; memcpy(&x, addr, sizeof x); return sg_from_bits(t, x); }
    case SG_I64: { int64_t x; memcpy(&x, addr, sizeof x); return sg_from_bits(t, (unsigned long long)x); }
    case SG_U64: { uint64_t x; memcpy(&x, addr, sizeof x); return sg_from_bits(t, x); }
    default: sg_fail("Load of a non-scalar member");
    }
    return r;
}

/* Store val, converted to type t, into typed memory. */
static void sg_write_scalar(void *addr, sg_ty t, sv val) {
    sv c = sg_cast(val, t);
    switch (t) {
    case SG_F64: memcpy(addr, &c.v.d, sizeof(double)); return;
    case SG_F32: { float x = (float)c.v.d; memcpy(addr, &x, sizeof x); return; }
    case SG_LD: memcpy(addr, &c.v.ld, sizeof(long double)); return;
    default: break;
    }
    unsigned long long bits = sg_bits_of(c);
    switch (t) {
    case SG_I8: case SG_U8: { uint8_t x = (uint8_t)bits; memcpy(addr, &x, sizeof x); return; }
    case SG_I16: case SG_U16: { uint16_t x = (uint16_t)bits; memcpy(addr, &x, sizeof x); return; }
    case SG_I32: case SG_U32: { uint32_t x = (uint32_t)bits; memcpy(addr, &x, sizeof x); return; }
    default: { uint64_t x = (uint64_t)bits; memcpy(addr, &x, sizeof x); return; }
    }
}

static sv sg_deref(sv p) {
    if (p.t != SG_PTR) sg_fail("Dereference of a non-pointer");
    const sg_ptr *q = &p.v.ptr;
    switch (q->kind) {
    case SG_PK_CELL: return *(sv *)q->addr;
    case SG_PK_SCALAR: return sg_read_scalar(q->addr, q->pt);
    case SG_PK_ARRAY:
        /* An array decays: its first element is a row, or a leaf for one dimension. */
        if (q->nd > 1) return sg_array_ptr(q->addr, q->ekind, q->pt, q->agg, q->dims + 1, q->nd - 1);
        return sg_ptr_value(q->addr, q->ekind, q->pt, q->agg);
    default: sg_fail("A struct has no single value; load a member");
    }
    return p;
}

static void sg_store_ptr(sv p, sv val) {
    if (p.t != SG_PTR) sg_fail("Store through a non-pointer");
    switch (p.v.ptr.kind) {
    case SG_PK_CELL: *(sv *)p.v.ptr.addr = val; return;
    case SG_PK_SCALAR: sg_write_scalar(p.v.ptr.addr, p.v.ptr.pt, val); return;
    default: sg_fail("A struct cannot be stored as a whole; store a member");
    }
}

/* `n.name`: a pointer to the named member of the struct that p addresses. */
static sv sg_member(sv p, const char *name) {
    if (p.t != SG_PTR || p.v.ptr.kind != SG_PK_AGG) sg_fail("Member access needs a pointer to a struct");
    const sg_agg *a = p.v.ptr.agg;
    for (int i = 0; i < a->nfields; i++) {
        const sg_field *f = &a->fields[i];
        if (strcmp(f->name, name) == 0) {
            return sg_ptr_value((char *)p.v.ptr.addr + f->off, f->kind, f->pt, f->agg);
        }
    }
    sg_fail("No such struct member");
    return p;
}

