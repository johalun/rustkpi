// /* Mutex used in the shared code */
// #define E1000_MUTEX                     struct mtx
// #define E1000_MUTEX_INIT(mutex)         mtx_init((mutex), #mutex, \
//                                             MTX_NETWORK_LOCK, \
// 					    MTX_DEF | MTX_DUPOK)
// #define E1000_MUTEX_DESTROY(mutex)      mtx_destroy(mutex)
// #define E1000_MUTEX_LOCK(mutex)         mtx_lock(mutex)
// #define E1000_MUTEX_TRYLOCK(mutex)      mtx_trylock(mutex)
// #define E1000_MUTEX_UNLOCK(mutex)       mtx_unlock(mutex)

// #define	mtx_init(m, n, t, o)	\
//   _mtx_init(&(m)->mtx_lock, n, t, o)
// #define	mtx_destroy(m)		\
//   _mtx_destroy(&(m)->mtx_lock)

// #define mtx_lock(m)		mtx_lock_flags((m), 0)
// #define mtx_unlock(m)		mtx_unlock_flags((m), 0)

// #define	_mtx_lock_flags(m, o, f, l)					\
// 	__mtx_lock_flags(&(m)->mtx_lock, o, f, l)
// #define	_mtx_unlock_flags(m, o, f, l)					\
// 	__mtx_unlock_flags(&(m)->mtx_lock, o, f, l)

#[macro_export]
macro_rules! e1000_mutex_init {
    ($m:tt) => {
        e1000_println!("e1000_mutex_init: {:?}", stringify!($m));
        mtx_init!($m,
                  cstr!(stringify!($m)),
                  kernel::sys::mutex_sys::MTX_NETWORK_LOCK as *const _ as *const i8,
                  kernel::sys::mutex_sys::MTX_DEF as i32 | kernel::sys::mutex_sys::MTX_DUPOK as i32);
    }
}

#[macro_export]
macro_rules! e1000_mutex_lock {
    ($m:tt) => {
        // e1000_verbose_println!("e1000_mutex_lock: {:?}", stringify!($m));
        mtx_lock!($m, 0);
    }
}

#[macro_export]
macro_rules! e1000_mutex_unlock {
    ($m:tt) => {
        // e1000_verbose_println!("e1000_mutex_unlock: {:?}", stringify!($m));
        mtx_unlock!($m, 0);
    }
}

#[macro_export]
macro_rules! mtx_init {
    ($m:tt, $n:expr, $o:expr, $t:expr) => {
        unsafe {
            kernel::sys::mutex_sys::_mtx_init(&mut $m.mtx_lock, $n, $o, $t);
        }
    }
}
// #define	mtx_lock_flags(m, opts)						\
// 	mtx_lock_flags_((m), (opts), LOCK_FILE, LOCK_LINE)

// void	__mtx_lock_flags(volatile uintptr_t *c, int opts, const char *file,
// 	    int line);

#[macro_export]
macro_rules! mtx_lock {
    ($m:tt, $f:expr) => {
        unsafe {
            kernel::sys::mutex_sys::__mtx_lock_flags(&mut $m.mtx_lock, $f, cstr!(file!()), line!() as i32);
        }
    }
}

#[macro_export]
macro_rules! mtx_unlock {
    ($m:tt, $f:expr) => {
        unsafe {
            kernel::sys::mutex_sys::__mtx_unlock_flags(&mut $m.mtx_lock, $f, cstr!(file!()), line!() as i32);
        }
    }
}

// All our locking now done in iflib so the above macros are unused.
#[macro_export]
macro_rules! assert_ctx_lock_held {
    ($m:tt) => {
        // Assert lock if INVARIANTS
        // We run benchmarks in -NODEBUG kernel so skip for now.
    }
}
