# prime-summer

The easiest way to try this is via Docker:

```console
$ docker run --rm anderskaseorg/prime-summer prime-summer 100
Sum of squares of primes ≤ 100 is 65796
$ docker run --rm anderskaseorg/prime-summer prime-summer 10000000000000
Sum of squares of primes ≤ 10000000000000 is 11262617785640702236670513970349205634
```

If you want to build it yourself, you’ll need Rust, GCC, and
[libprimesieve](https://github.com/kimwalisch/primesieve).
