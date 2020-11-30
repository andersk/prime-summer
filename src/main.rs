use primesieve_sys::{
    primesieve_free, primesieve_free_iterator, primesieve_generate_primes, primesieve_init,
    primesieve_iterator, primesieve_next_prime, primesieve_prev_prime, primesieve_skipto,
    UINT64_PRIMES,
};
use rug::ops::Pow;
use rug::Integer;
use std::collections::VecDeque;
use std::env;
use std::error::Error;
use std::mem;
use std::slice;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Query {
    x: u64,
    i: usize,
    sign: bool,
    w: u64,
}

fn phi(x: u64, primes: &[u64], sign: bool, w: u64, y: u64, queries: &mut Vec<Query>) -> Integer {
    if x >= y || primes.is_empty() {
        let mut o: Integer = Integer::from(x) * (x + 1) * (2 * x + 1) / 6 * w * w;
        if sign {
            o = -o;
        }
        for (i, &p) in primes.iter().enumerate() {
            o += phi(x / p, &primes[..i], !sign, w * p, y, queries);
        }
        o
    } else {
        queries.push(Query {
            x,
            i: primes.len(),
            sign,
            w,
        });
        Integer::new()
    }
}

fn sum_primes_squared(n: u64) -> Integer {
    if n == 0 {
        return Integer::new();
    }
    let cbrt_n: u64 = Integer::from(n).root(3).to_u64().unwrap();
    let sqrt_n: u64 = Integer::from(n).sqrt().to_u64().unwrap();
    let small_primes = unsafe {
        let mut small_primes_size = 0;
        let small_primes_buf =
            primesieve_generate_primes(2, cbrt_n, &mut small_primes_size, UINT64_PRIMES);
        let small_primes: Box<[u64]> =
            slice::from_raw_parts(small_primes_buf as *const u64, small_primes_size).into();
        primesieve_free(small_primes_buf);
        small_primes
    };

    let y = sqrt_n * 3;
    let mut queries = Vec::new();
    let mut ret = phi(n, &small_primes, false, 1, y, &mut queries);

    queries.sort_by_key(|query| (query.x, query.i));
    let mut queries = queries.into_iter();
    if let Some(mut query) = queries.next() {
        let base = 1
            << mem::size_of_val(&small_primes.len()) as u32 * 8
                - small_primes.len().leading_zeros();
        let mut accumulator = Vec::new();
        accumulator.resize_with(base + small_primes.len() + 1, Integer::new);
        let mut queue = VecDeque::new();
        queue.resize((small_primes.last().unwrap_or(&0) + 1) as usize, !0);
        for (i, &p) in small_primes.iter().enumerate() {
            queue[p as usize] = i;
        }
        let mut x = 0;
        'outer: loop {
            let i = match queue.pop_front() {
                Some(i) if i != !0 => {
                    let mut k = i;
                    let mut j = small_primes[k] as usize - 1;
                    while j < queue.len() && queue[j] != !0 {
                        if queue[j] > k {
                            mem::swap(&mut k, &mut queue[j]);
                        }
                        j += small_primes[k] as usize;
                    }
                    if j >= queue.len() {
                        queue.resize(j + 1, !0);
                    }
                    queue[j] = k;
                    i
                }
                _ => small_primes.len(),
            };
            let x2 = Integer::from(x).pow(2);
            let mut node = base + i;
            while node != 0 {
                accumulator[node] += &x2;
                node >>= 1;
            }
            while query.x == x {
                let mut node = base + query.i;
                let mut reply = accumulator[node].clone();
                while node != 0 {
                    if node & 1 == 0 {
                        reply += &accumulator[node + 1];
                    }
                    node >>= 1;
                }
                if query.sign {
                    reply = -reply;
                }
                ret += reply * query.w * query.w;
                if let Some(query1) = queries.next() {
                    query = query1;
                } else {
                    break 'outer;
                }
            }
            x += 1;
        }
    }

    ret -= 1;
    for &p in &*small_primes {
        ret += p * p;
    }
    let mut pi: primesieve_iterator;
    let mut qi: primesieve_iterator;
    unsafe {
        pi = mem::zeroed();
        primesieve_init(&mut pi);
        primesieve_skipto(&mut pi, sqrt_n + 1, cbrt_n);
        qi = mem::zeroed();
        primesieve_init(&mut qi);
        primesieve_skipto(&mut qi, sqrt_n, n / cbrt_n);
    }
    let mut p;
    let mut q = unsafe { primesieve_next_prime(&mut qi) };
    let mut s = Integer::new();
    while {
        p = unsafe { primesieve_prev_prime(&mut pi) };
        p > cbrt_n
    } {
        let p2 = p * p;
        s += p2;
        while p * q <= n {
            s += Integer::from(q).pow(2);
            q = unsafe { primesieve_next_prime(&mut qi) };
        }
        ret -= &s * p2;
    }

    unsafe {
        primesieve_free_iterator(&mut pi);
        primesieve_free_iterator(&mut qi);
        drop(pi);
    }

    ret
}

fn main() -> Result<(), Box<dyn Error>> {
    if let [_, n] = &*env::args().collect::<Vec<_>>() {
        let n = n.parse()?;
        println!(
            "Sum of squares of primes ≤ {} is {}",
            n,
            sum_primes_squared(n)
        );
    } else {
        Err("Usage: prime-summer N")?;
    }
    Ok(())
}

#[test]
fn test_small() {
    assert_eq!(sum_primes_squared(0), 0);
    assert_eq!(sum_primes_squared(1), 0);
    let mut s = 0;
    for n in 2..10001 {
        let mut i = 2;
        loop {
            if i * i > n {
                s += n * n;
                break;
            }
            if n % i == 0 {
                break;
            }
            i += 1;
        }
        assert_eq!(sum_primes_squared(n), s);
    }
}

#[test]
fn test_powers_of_10() {
    assert_eq!(sum_primes_squared(2), "4".parse::<Integer>().unwrap());
    assert_eq!(sum_primes_squared(29), "2397".parse::<Integer>().unwrap());
    assert_eq!(
        sum_primes_squared(541),
        "8384727".parse::<Integer>().unwrap()
    );
    assert_eq!(
        sum_primes_squared(7919),
        "19053119163".parse::<Integer>().unwrap()
    );
    assert_eq!(
        sum_primes_squared(104729),
        "34099597499091".parse::<Integer>().unwrap()
    );
    assert_eq!(
        sum_primes_squared(1299709),
        "53251529659694763".parse::<Integer>().unwrap()
    );
    assert_eq!(
        sum_primes_squared(15485863),
        "76304519151822049179".parse::<Integer>().unwrap()
    );
    assert_eq!(
        sum_primes_squared(179424673),
        "103158861357874372432083".parse::<Integer>().unwrap()
    );
    assert_eq!(
        sum_primes_squared(2038074743),
        "133759354162117403400944283".parse::<Integer>().unwrap()
    );
    assert_eq!(
        sum_primes_squared(22801763489),
        "168072405102068540986037048787".parse::<Integer>().unwrap()
    );
    assert_eq!(
        sum_primes_squared(252097800623),
        "206076219788796447007218742841043"
            .parse::<Integer>()
            .unwrap()
    );
    assert_eq!(
        sum_primes_squared(2760727302517),
        "247754953701579144582110673365391267"
            .parse::<Integer>()
            .unwrap()
    );
}
