#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi;
use std::os::raw::*;

pub const _UNISTD_H: u32 = 1;
pub const _FEATURES_H: u32 = 1;
pub const _ISOC95_SOURCE: u32 = 1;
pub const _ISOC99_SOURCE: u32 = 1;
pub const _ISOC11_SOURCE: u32 = 1;
pub const _POSIX_SOURCE: u32 = 1;
pub const _POSIX_C_SOURCE: u32 = 200809;
pub const _XOPEN_SOURCE: u32 = 700;
pub const _XOPEN_SOURCE_EXTENDED: u32 = 1;
pub const _LARGEFILE64_SOURCE: u32 = 1;
pub const _DEFAULT_SOURCE: u32 = 1;
pub const _ATFILE_SOURCE: u32 = 1;
pub const __USE_ISOC11: u32 = 1;
pub const __USE_ISOC99: u32 = 1;
pub const __USE_ISOC95: u32 = 1;
pub const __USE_ISOCXX11: u32 = 1;
pub const __USE_POSIX: u32 = 1;
pub const __USE_POSIX2: u32 = 1;
pub const __USE_POSIX199309: u32 = 1;
pub const __USE_POSIX199506: u32 = 1;
pub const __USE_XOPEN2K: u32 = 1;
pub const __USE_XOPEN2K8: u32 = 1;
pub const __USE_XOPEN: u32 = 1;
pub const __USE_XOPEN_EXTENDED: u32 = 1;
pub const __USE_UNIX98: u32 = 1;
pub const _LARGEFILE_SOURCE: u32 = 1;
pub const __USE_XOPEN2K8XSI: u32 = 1;
pub const __USE_XOPEN2KXSI: u32 = 1;
pub const __USE_LARGEFILE: u32 = 1;
pub const __USE_LARGEFILE64: u32 = 1;
pub const __USE_MISC: u32 = 1;
pub const __USE_ATFILE: u32 = 1;
pub const __USE_GNU: u32 = 1;
pub const __USE_FORTIFY_LEVEL: u32 = 0;
pub const __GLIBC_USE_DEPRECATED_GETS: u32 = 0;
pub const _STDC_PREDEF_H: u32 = 1;
pub const __STDC_IEC_559__: u32 = 1;
pub const __STDC_IEC_559_COMPLEX__: u32 = 1;
pub const __STDC_ISO_10646__: u32 = 201706;
pub const __STDC_NO_THREADS__: u32 = 1;
pub const __GNU_LIBRARY__: u32 = 6;
pub const __GLIBC__: u32 = 2;
pub const __GLIBC_MINOR__: u32 = 27;
pub const _SYS_CDEFS_H: u32 = 1;
pub const __glibc_c99_flexarr_available: u32 = 1;
pub const __WORDSIZE: u32 = 64;
pub const __WORDSIZE_TIME64_COMPAT32: u32 = 1;
pub const __SYSCALL_WORDSIZE: u32 = 64;
pub const __HAVE_GENERIC_SELECTION: u32 = 0;
pub const _POSIX_VERSION: u32 = 200809;
pub const __POSIX2_THIS_VERSION: u32 = 200809;
pub const _POSIX2_VERSION: u32 = 200809;
pub const _POSIX2_C_VERSION: u32 = 200809;
pub const _POSIX2_C_BIND: u32 = 200809;
pub const _POSIX2_C_DEV: u32 = 200809;
pub const _POSIX2_SW_DEV: u32 = 200809;
pub const _POSIX2_LOCALEDEF: u32 = 200809;
pub const _XOPEN_VERSION: u32 = 700;
pub const _XOPEN_XCU_VERSION: u32 = 4;
pub const _XOPEN_XPG2: u32 = 1;
pub const _XOPEN_XPG3: u32 = 1;
pub const _XOPEN_XPG4: u32 = 1;
pub const _XOPEN_UNIX: u32 = 1;
pub const _XOPEN_CRYPT: u32 = 1;
pub const _XOPEN_ENH_I18N: u32 = 1;
pub const _XOPEN_LEGACY: u32 = 1;
pub const _BITS_POSIX_OPT_H: u32 = 1;
pub const _POSIX_JOB_CONTROL: u32 = 1;
pub const _POSIX_SAVED_IDS: u32 = 1;
pub const _POSIX_PRIORITY_SCHEDULING: u32 = 200809;
pub const _POSIX_SYNCHRONIZED_IO: u32 = 200809;
pub const _POSIX_FSYNC: u32 = 200809;
pub const _POSIX_MAPPED_FILES: u32 = 200809;
pub const _POSIX_MEMLOCK: u32 = 200809;
pub const _POSIX_MEMLOCK_RANGE: u32 = 200809;
pub const _POSIX_MEMORY_PROTECTION: u32 = 200809;
pub const _POSIX_CHOWN_RESTRICTED: u32 = 0;
pub const _POSIX_VDISABLE: u8 = 0u8;
pub const _POSIX_NO_TRUNC: u32 = 1;
pub const _XOPEN_REALTIME: u32 = 1;
pub const _XOPEN_REALTIME_THREADS: u32 = 1;
pub const _XOPEN_SHM: u32 = 1;
pub const _POSIX_THREADS: u32 = 200809;
pub const _POSIX_REENTRANT_FUNCTIONS: u32 = 1;
pub const _POSIX_THREAD_SAFE_FUNCTIONS: u32 = 200809;
pub const _POSIX_THREAD_PRIORITY_SCHEDULING: u32 = 200809;
pub const _POSIX_THREAD_ATTR_STACKSIZE: u32 = 200809;
pub const _POSIX_THREAD_ATTR_STACKADDR: u32 = 200809;
pub const _POSIX_THREAD_PRIO_INHERIT: u32 = 200809;
pub const _POSIX_THREAD_PRIO_PROTECT: u32 = 200809;
pub const _POSIX_THREAD_ROBUST_PRIO_INHERIT: u32 = 200809;
pub const _POSIX_THREAD_ROBUST_PRIO_PROTECT: i32 = -1;
pub const _POSIX_SEMAPHORES: u32 = 200809;
pub const _POSIX_REALTIME_SIGNALS: u32 = 200809;
pub const _POSIX_ASYNCHRONOUS_IO: u32 = 200809;
pub const _POSIX_ASYNC_IO: u32 = 1;
pub const _LFS_ASYNCHRONOUS_IO: u32 = 1;
pub const _POSIX_PRIORITIZED_IO: u32 = 200809;
pub const _LFS64_ASYNCHRONOUS_IO: u32 = 1;
pub const _LFS_LARGEFILE: u32 = 1;
pub const _LFS64_LARGEFILE: u32 = 1;
pub const _LFS64_STDIO: u32 = 1;
pub const _POSIX_SHARED_MEMORY_OBJECTS: u32 = 200809;
pub const _POSIX_CPUTIME: u32 = 0;
pub const _POSIX_THREAD_CPUTIME: u32 = 0;
pub const _POSIX_REGEXP: u32 = 1;
pub const _POSIX_READER_WRITER_LOCKS: u32 = 200809;
pub const _POSIX_SHELL: u32 = 1;
pub const _POSIX_TIMEOUTS: u32 = 200809;
pub const _POSIX_SPIN_LOCKS: u32 = 200809;
pub const _POSIX_SPAWN: u32 = 200809;
pub const _POSIX_TIMERS: u32 = 200809;
pub const _POSIX_BARRIERS: u32 = 200809;
pub const _POSIX_MESSAGE_PASSING: u32 = 200809;
pub const _POSIX_THREAD_PROCESS_SHARED: u32 = 200809;
pub const _POSIX_MONOTONIC_CLOCK: u32 = 0;
pub const _POSIX_CLOCK_SELECTION: u32 = 200809;
pub const _POSIX_ADVISORY_INFO: u32 = 200809;
pub const _POSIX_IPV6: u32 = 200809;
pub const _POSIX_RAW_SOCKETS: u32 = 200809;
pub const _POSIX2_CHAR_TERM: u32 = 200809;
pub const _POSIX_SPORADIC_SERVER: i32 = -1;
pub const _POSIX_THREAD_SPORADIC_SERVER: i32 = -1;
pub const _POSIX_TRACE: i32 = -1;
pub const _POSIX_TRACE_EVENT_FILTER: i32 = -1;
pub const _POSIX_TRACE_INHERIT: i32 = -1;
pub const _POSIX_TRACE_LOG: i32 = -1;
pub const _POSIX_TYPED_MEMORY_OBJECTS: i32 = -1;
pub const _POSIX_V7_LPBIG_OFFBIG: i32 = -1;
pub const _POSIX_V6_LPBIG_OFFBIG: i32 = -1;
pub const _XBS5_LPBIG_OFFBIG: i32 = -1;
pub const _POSIX_V7_LP64_OFF64: u32 = 1;
pub const _POSIX_V6_LP64_OFF64: u32 = 1;
pub const _XBS5_LP64_OFF64: u32 = 1;
pub const __ILP32_OFF32_CFLAGS: &[u8; 5usize] = b"-m32\0";
pub const __ILP32_OFF32_LDFLAGS: &[u8; 5usize] = b"-m32\0";
pub const __ILP32_OFFBIG_CFLAGS: &[u8; 48usize] =
    b"-m32 -D_LARGEFILE_SOURCE -D_FILE_OFFSET_BITS=64\0";
pub const __ILP32_OFFBIG_LDFLAGS: &[u8; 5usize] = b"-m32\0";
pub const __LP64_OFF64_CFLAGS: &[u8; 5usize] = b"-m64\0";
pub const __LP64_OFF64_LDFLAGS: &[u8; 5usize] = b"-m64\0";
pub const STDIN_FILENO: u32 = 0;
pub const STDOUT_FILENO: u32 = 1;
pub const STDERR_FILENO: u32 = 2;
pub const _BITS_TYPES_H: u32 = 1;
pub const _BITS_TYPESIZES_H: u32 = 1;
pub const __OFF_T_MATCHES_OFF64_T: u32 = 1;
pub const __INO_T_MATCHES_INO64_T: u32 = 1;
pub const __RLIM_T_MATCHES_RLIM64_T: u32 = 1;
pub const __FD_SETSIZE: u32 = 1024;
pub const R_OK: u32 = 4;
pub const W_OK: u32 = 2;
pub const X_OK: u32 = 1;
pub const F_OK: u32 = 0;
pub const SEEK_SET: u32 = 0;
pub const SEEK_CUR: u32 = 1;
pub const SEEK_END: u32 = 2;
pub const SEEK_DATA: u32 = 3;
pub const SEEK_HOLE: u32 = 4;
pub const L_SET: u32 = 0;
pub const L_INCR: u32 = 1;
pub const L_XTND: u32 = 2;
pub const _GETOPT_POSIX_H: u32 = 1;
pub const _GETOPT_CORE_H: u32 = 1;
pub const F_ULOCK: u32 = 0;
pub const F_LOCK: u32 = 1;
pub const F_TLOCK: u32 = 2;
pub const F_TEST: u32 = 3;
pub const DDS_VERSION: u32 = 20900;
pub const DDS_HANDS: u32 = 4;
pub const DDS_SUITS: u32 = 4;
pub const DDS_STRAINS: u32 = 5;
pub const MAXNOOFBOARDS: u32 = 200;
pub const MAXNOOFTABLES: u32 = 40;
pub const RETURN_NO_FAULT: u32 = 1;
pub const TEXT_NO_FAULT: &[u8; 8usize] = b"Success\0";
pub const RETURN_UNKNOWN_FAULT: i32 = -1;
pub const TEXT_UNKNOWN_FAULT: &[u8; 14usize] = b"General error\0";
pub const RETURN_ZERO_CARDS: i32 = -2;
pub const TEXT_ZERO_CARDS: &[u8; 11usize] = b"Zero cards\0";
pub const RETURN_TARGET_TOO_HIGH: i32 = -3;
pub const TEXT_TARGET_TOO_HIGH: &[u8; 32usize] = b"Target exceeds number of tricks\0";
pub const RETURN_DUPLICATE_CARDS: i32 = -4;
pub const TEXT_DUPLICATE_CARDS: &[u8; 17usize] = b"Cards duplicated\0";
pub const RETURN_TARGET_WRONG_LO: i32 = -5;
pub const TEXT_TARGET_WRONG_LO: &[u8; 23usize] = b"Target is less than -1\0";
pub const RETURN_TARGET_WRONG_HI: i32 = -7;
pub const TEXT_TARGET_WRONG_HI: &[u8; 25usize] = b"Target is higher than 13\0";
pub const RETURN_SOLNS_WRONG_LO: i32 = -8;
pub const TEXT_SOLNS_WRONG_LO: &[u8; 35usize] = b"Solutions parameter is less than 1\0";
pub const RETURN_SOLNS_WRONG_HI: i32 = -9;
pub const TEXT_SOLNS_WRONG_HI: &[u8; 37usize] = b"Solutions parameter is higher than 3\0";
pub const RETURN_TOO_MANY_CARDS: i32 = -10;
pub const TEXT_TOO_MANY_CARDS: &[u8; 15usize] = b"Too many cards\0";
pub const RETURN_SUIT_OR_RANK: i32 = -12;
pub const TEXT_SUIT_OR_RANK: &[u8; 52usize] =
    b"currentTrickSuit or currentTrickRank has wrong data\0";
pub const RETURN_PLAYED_CARD: i32 = -13;
pub const TEXT_PLAYED_CARD: &[u8; 35usize] = b"Played card also remains in a hand\0";
pub const RETURN_CARD_COUNT: i32 = -14;
pub const TEXT_CARD_COUNT: &[u8; 42usize] = b"Wrong number of remaining cards in a hand\0";
pub const RETURN_THREAD_INDEX: i32 = -15;
pub const TEXT_THREAD_INDEX: &[u8; 33usize] = b"Thread index is not 0 .. maximum\0";
pub const RETURN_MODE_WRONG_LO: i32 = -16;
pub const TEXT_MODE_WRONG_LO: &[u8; 30usize] = b"Mode parameter is less than 0\0";
pub const RETURN_MODE_WRONG_HI: i32 = -17;
pub const TEXT_MODE_WRONG_HI: &[u8; 32usize] = b"Mode parameter is higher than 2\0";
pub const RETURN_TRUMP_WRONG: i32 = -18;
pub const TEXT_TRUMP_WRONG: &[u8; 23usize] = b"Trump is not in 0 .. 4\0";
pub const RETURN_FIRST_WRONG: i32 = -19;
pub const TEXT_FIRST_WRONG: &[u8; 23usize] = b"First is not in 0 .. 2\0";
pub const RETURN_PLAY_FAULT: i32 = -98;
pub const TEXT_PLAY_FAULT: &[u8; 24usize] = b"AnalysePlay input error\0";
pub const RETURN_PBN_FAULT: i32 = -99;
pub const TEXT_PBN_FAULT: &[u8; 17usize] = b"PBN string error\0";
pub const RETURN_TOO_MANY_BOARDS: i32 = -101;
pub const TEXT_TOO_MANY_BOARDS: &[u8; 26usize] = b"Too many boards requested\0";
pub const RETURN_THREAD_CREATE: i32 = -102;
pub const TEXT_THREAD_CREATE: &[u8; 25usize] = b"Could not create threads\0";
pub const RETURN_THREAD_WAIT: i32 = -103;
pub const TEXT_THREAD_WAIT: &[u8; 43usize] = b"Something failed waiting for thread to end\0";
pub const RETURN_THREAD_MISSING: i32 = -104;
pub const TEXT_THREAD_MISSING: &[u8; 35usize] = b"Multi-threading system not present\0";
pub const RETURN_NO_SUIT: i32 = -201;
pub const TEXT_NO_SUIT: &[u8; 42usize] = b"Denomination filter vector has no entries\0";
pub const RETURN_TOO_MANY_TABLES: i32 = -202;
pub const TEXT_TOO_MANY_TABLES: &[u8; 29usize] = b"Too many DD tables requested\0";
pub const RETURN_CHUNK_SIZE: i32 = -301;
pub const TEXT_CHUNK_SIZE: &[u8; 26usize] = b"Chunk size is less than 1\0";
pub const THREADMEM_SMALL_MAX_MB: u32 = 30;
pub const THREADMEM_SMALL_DEF_MB: u32 = 20;
pub const THREADMEM_LARGE_MAX_MB: u32 = 160;
pub const THREADMEM_LARGE_DEF_MB: u32 = 95;
pub const MAXNODE: u32 = 1;
pub const MINNODE: u32 = 0;
pub const SIMILARDEALLIMIT: u32 = 5;
pub const SIMILARMAXWINNODES: u32 = 700000;
pub const DDS_NOTRUMP: u32 = 4;
pub type __u_char = ::std::os::raw::c_uchar;
pub type __u_short = ::std::os::raw::c_ushort;
pub type __u_int = ::std::os::raw::c_uint;
pub type __u_long = ::std::os::raw::c_ulong;
pub type __int8_t = ::std::os::raw::c_schar;
pub type __uint8_t = ::std::os::raw::c_uchar;
pub type __int16_t = ::std::os::raw::c_short;
pub type __uint16_t = ::std::os::raw::c_ushort;
pub type __int32_t = ::std::os::raw::c_int;
pub type __uint32_t = ::std::os::raw::c_uint;
pub type __int64_t = ::std::os::raw::c_long;
pub type __uint64_t = ::std::os::raw::c_ulong;
pub type __quad_t = ::std::os::raw::c_long;
pub type __u_quad_t = ::std::os::raw::c_ulong;
pub type __intmax_t = ::std::os::raw::c_long;
pub type __uintmax_t = ::std::os::raw::c_ulong;
pub type __dev_t = ::std::os::raw::c_ulong;
pub type __uid_t = ::std::os::raw::c_uint;
pub type __gid_t = ::std::os::raw::c_uint;
pub type __ino_t = ::std::os::raw::c_ulong;
pub type __ino64_t = ::std::os::raw::c_ulong;
pub type __mode_t = ::std::os::raw::c_uint;
pub type __nlink_t = ::std::os::raw::c_ulong;
pub type __off_t = ::std::os::raw::c_long;
pub type __off64_t = ::std::os::raw::c_long;
pub type __pid_t = ::std::os::raw::c_int;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __fsid_t {
    pub __val: [::std::os::raw::c_int; 2usize],
}
#[test]
fn bindgen_test_layout___fsid_t() {
    assert_eq!(
        ::std::mem::size_of::<__fsid_t>(),
        8usize,
        concat!("Size of: ", stringify!(__fsid_t))
    );
    assert_eq!(
        ::std::mem::align_of::<__fsid_t>(),
        4usize,
        concat!("Alignment of ", stringify!(__fsid_t))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<__fsid_t>())).__val as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(__fsid_t),
            "::",
            stringify!(__val)
        )
    );
}
pub type __clock_t = ::std::os::raw::c_long;
pub type __rlim_t = ::std::os::raw::c_ulong;
pub type __rlim64_t = ::std::os::raw::c_ulong;
pub type __id_t = ::std::os::raw::c_uint;
pub type __time_t = ::std::os::raw::c_long;
pub type __useconds_t = ::std::os::raw::c_uint;
pub type __suseconds_t = ::std::os::raw::c_long;
pub type __daddr_t = ::std::os::raw::c_int;
pub type __key_t = ::std::os::raw::c_int;
pub type __clockid_t = ::std::os::raw::c_int;
pub type __timer_t = *mut ::std::os::raw::c_void;
pub type __blksize_t = ::std::os::raw::c_long;
pub type __blkcnt_t = ::std::os::raw::c_long;
pub type __blkcnt64_t = ::std::os::raw::c_long;
pub type __fsblkcnt_t = ::std::os::raw::c_ulong;
pub type __fsblkcnt64_t = ::std::os::raw::c_ulong;
pub type __fsfilcnt_t = ::std::os::raw::c_ulong;
pub type __fsfilcnt64_t = ::std::os::raw::c_ulong;
pub type __fsword_t = ::std::os::raw::c_long;
pub type __ssize_t = ::std::os::raw::c_long;
pub type __syscall_slong_t = ::std::os::raw::c_long;
pub type __syscall_ulong_t = ::std::os::raw::c_ulong;
pub type __loff_t = __off64_t;
pub type __caddr_t = *mut ::std::os::raw::c_char;
pub type __intptr_t = ::std::os::raw::c_long;
pub type __socklen_t = ::std::os::raw::c_uint;
pub type __sig_atomic_t = ::std::os::raw::c_int;
pub type ssize_t = __ssize_t;
pub type size_t = ::std::os::raw::c_ulong;
pub type gid_t = __gid_t;
pub type uid_t = __uid_t;
pub type off_t = __off_t;
pub type off64_t = __off64_t;
pub type useconds_t = __useconds_t;
pub type pid_t = __pid_t;
pub type socklen_t = __socklen_t;
extern "C" {
    pub fn access(
        __name: *const ::std::os::raw::c_char,
        __type: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn euidaccess(
        __name: *const ::std::os::raw::c_char,
        __type: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn eaccess(
        __name: *const ::std::os::raw::c_char,
        __type: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn faccessat(
        __fd: ::std::os::raw::c_int,
        __file: *const ::std::os::raw::c_char,
        __type: ::std::os::raw::c_int,
        __flag: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn lseek(
        __fd: ::std::os::raw::c_int,
        __offset: __off_t,
        __whence: ::std::os::raw::c_int,
    ) -> __off_t;
}
extern "C" {
    pub fn lseek64(
        __fd: ::std::os::raw::c_int,
        __offset: __off64_t,
        __whence: ::std::os::raw::c_int,
    ) -> __off64_t;
}
extern "C" {
    pub fn close(__fd: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn read(
        __fd: ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_void,
        __nbytes: size_t,
    ) -> ssize_t;
}
extern "C" {
    pub fn write(
        __fd: ::std::os::raw::c_int,
        __buf: *const ::std::os::raw::c_void,
        __n: size_t,
    ) -> ssize_t;
}
extern "C" {
    pub fn pread(
        __fd: ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_void,
        __nbytes: size_t,
        __offset: __off_t,
    ) -> ssize_t;
}
extern "C" {
    pub fn pwrite(
        __fd: ::std::os::raw::c_int,
        __buf: *const ::std::os::raw::c_void,
        __n: size_t,
        __offset: __off_t,
    ) -> ssize_t;
}
extern "C" {
    pub fn pread64(
        __fd: ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_void,
        __nbytes: size_t,
        __offset: __off64_t,
    ) -> ssize_t;
}
extern "C" {
    pub fn pwrite64(
        __fd: ::std::os::raw::c_int,
        __buf: *const ::std::os::raw::c_void,
        __n: size_t,
        __offset: __off64_t,
    ) -> ssize_t;
}
extern "C" {
    pub fn pipe(__pipedes: *mut ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn pipe2(
        __pipedes: *mut ::std::os::raw::c_int,
        __flags: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn alarm(__seconds: ::std::os::raw::c_uint) -> ::std::os::raw::c_uint;
}
extern "C" {
    pub fn sleep(__seconds: ::std::os::raw::c_uint) -> ::std::os::raw::c_uint;
}
extern "C" {
    pub fn ualarm(__value: __useconds_t, __interval: __useconds_t) -> __useconds_t;
}
extern "C" {
    pub fn usleep(__useconds: __useconds_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn pause() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn chown(
        __file: *const ::std::os::raw::c_char,
        __owner: __uid_t,
        __group: __gid_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn fchown(
        __fd: ::std::os::raw::c_int,
        __owner: __uid_t,
        __group: __gid_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn lchown(
        __file: *const ::std::os::raw::c_char,
        __owner: __uid_t,
        __group: __gid_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn fchownat(
        __fd: ::std::os::raw::c_int,
        __file: *const ::std::os::raw::c_char,
        __owner: __uid_t,
        __group: __gid_t,
        __flag: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn chdir(__path: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn fchdir(__fd: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn getcwd(
        __buf: *mut ::std::os::raw::c_char,
        __size: size_t,
    ) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn get_current_dir_name() -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn getwd(__buf: *mut ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn dup(__fd: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn dup2(__fd: ::std::os::raw::c_int, __fd2: ::std::os::raw::c_int)
        -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn dup3(
        __fd: ::std::os::raw::c_int,
        __fd2: ::std::os::raw::c_int,
        __flags: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub static mut __environ: *mut *mut ::std::os::raw::c_char;
}
extern "C" {
    pub static mut environ: *mut *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn execve(
        __path: *const ::std::os::raw::c_char,
        __argv: *const *mut ::std::os::raw::c_char,
        __envp: *const *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn fexecve(
        __fd: ::std::os::raw::c_int,
        __argv: *const *mut ::std::os::raw::c_char,
        __envp: *const *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn execv(
        __path: *const ::std::os::raw::c_char,
        __argv: *const *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn execle(
        __path: *const ::std::os::raw::c_char,
        __arg: *const ::std::os::raw::c_char,
        ...
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn execl(
        __path: *const ::std::os::raw::c_char,
        __arg: *const ::std::os::raw::c_char,
        ...
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn execvp(
        __file: *const ::std::os::raw::c_char,
        __argv: *const *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn execlp(
        __file: *const ::std::os::raw::c_char,
        __arg: *const ::std::os::raw::c_char,
        ...
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn execvpe(
        __file: *const ::std::os::raw::c_char,
        __argv: *const *mut ::std::os::raw::c_char,
        __envp: *const *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn nice(__inc: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn _exit(__status: ::std::os::raw::c_int);
}
pub const _PC_LINK_MAX: ::std::os::raw::c_uint = 0;
pub const _PC_MAX_CANON: ::std::os::raw::c_uint = 1;
pub const _PC_MAX_INPUT: ::std::os::raw::c_uint = 2;
pub const _PC_NAME_MAX: ::std::os::raw::c_uint = 3;
pub const _PC_PATH_MAX: ::std::os::raw::c_uint = 4;
pub const _PC_PIPE_BUF: ::std::os::raw::c_uint = 5;
pub const _PC_CHOWN_RESTRICTED: ::std::os::raw::c_uint = 6;
pub const _PC_NO_TRUNC: ::std::os::raw::c_uint = 7;
pub const _PC_VDISABLE: ::std::os::raw::c_uint = 8;
pub const _PC_SYNC_IO: ::std::os::raw::c_uint = 9;
pub const _PC_ASYNC_IO: ::std::os::raw::c_uint = 10;
pub const _PC_PRIO_IO: ::std::os::raw::c_uint = 11;
pub const _PC_SOCK_MAXBUF: ::std::os::raw::c_uint = 12;
pub const _PC_FILESIZEBITS: ::std::os::raw::c_uint = 13;
pub const _PC_REC_INCR_XFER_SIZE: ::std::os::raw::c_uint = 14;
pub const _PC_REC_MAX_XFER_SIZE: ::std::os::raw::c_uint = 15;
pub const _PC_REC_MIN_XFER_SIZE: ::std::os::raw::c_uint = 16;
pub const _PC_REC_XFER_ALIGN: ::std::os::raw::c_uint = 17;
pub const _PC_ALLOC_SIZE_MIN: ::std::os::raw::c_uint = 18;
pub const _PC_SYMLINK_MAX: ::std::os::raw::c_uint = 19;
pub const _PC_2_SYMLINKS: ::std::os::raw::c_uint = 20;
pub type _bindgen_ty_1 = ::std::os::raw::c_uint;
pub const _SC_ARG_MAX: ::std::os::raw::c_uint = 0;
pub const _SC_CHILD_MAX: ::std::os::raw::c_uint = 1;
pub const _SC_CLK_TCK: ::std::os::raw::c_uint = 2;
pub const _SC_NGROUPS_MAX: ::std::os::raw::c_uint = 3;
pub const _SC_OPEN_MAX: ::std::os::raw::c_uint = 4;
pub const _SC_STREAM_MAX: ::std::os::raw::c_uint = 5;
pub const _SC_TZNAME_MAX: ::std::os::raw::c_uint = 6;
pub const _SC_JOB_CONTROL: ::std::os::raw::c_uint = 7;
pub const _SC_SAVED_IDS: ::std::os::raw::c_uint = 8;
pub const _SC_REALTIME_SIGNALS: ::std::os::raw::c_uint = 9;
pub const _SC_PRIORITY_SCHEDULING: ::std::os::raw::c_uint = 10;
pub const _SC_TIMERS: ::std::os::raw::c_uint = 11;
pub const _SC_ASYNCHRONOUS_IO: ::std::os::raw::c_uint = 12;
pub const _SC_PRIORITIZED_IO: ::std::os::raw::c_uint = 13;
pub const _SC_SYNCHRONIZED_IO: ::std::os::raw::c_uint = 14;
pub const _SC_FSYNC: ::std::os::raw::c_uint = 15;
pub const _SC_MAPPED_FILES: ::std::os::raw::c_uint = 16;
pub const _SC_MEMLOCK: ::std::os::raw::c_uint = 17;
pub const _SC_MEMLOCK_RANGE: ::std::os::raw::c_uint = 18;
pub const _SC_MEMORY_PROTECTION: ::std::os::raw::c_uint = 19;
pub const _SC_MESSAGE_PASSING: ::std::os::raw::c_uint = 20;
pub const _SC_SEMAPHORES: ::std::os::raw::c_uint = 21;
pub const _SC_SHARED_MEMORY_OBJECTS: ::std::os::raw::c_uint = 22;
pub const _SC_AIO_LISTIO_MAX: ::std::os::raw::c_uint = 23;
pub const _SC_AIO_MAX: ::std::os::raw::c_uint = 24;
pub const _SC_AIO_PRIO_DELTA_MAX: ::std::os::raw::c_uint = 25;
pub const _SC_DELAYTIMER_MAX: ::std::os::raw::c_uint = 26;
pub const _SC_MQ_OPEN_MAX: ::std::os::raw::c_uint = 27;
pub const _SC_MQ_PRIO_MAX: ::std::os::raw::c_uint = 28;
pub const _SC_VERSION: ::std::os::raw::c_uint = 29;
pub const _SC_PAGESIZE: ::std::os::raw::c_uint = 30;
pub const _SC_RTSIG_MAX: ::std::os::raw::c_uint = 31;
pub const _SC_SEM_NSEMS_MAX: ::std::os::raw::c_uint = 32;
pub const _SC_SEM_VALUE_MAX: ::std::os::raw::c_uint = 33;
pub const _SC_SIGQUEUE_MAX: ::std::os::raw::c_uint = 34;
pub const _SC_TIMER_MAX: ::std::os::raw::c_uint = 35;
pub const _SC_BC_BASE_MAX: ::std::os::raw::c_uint = 36;
pub const _SC_BC_DIM_MAX: ::std::os::raw::c_uint = 37;
pub const _SC_BC_SCALE_MAX: ::std::os::raw::c_uint = 38;
pub const _SC_BC_STRING_MAX: ::std::os::raw::c_uint = 39;
pub const _SC_COLL_WEIGHTS_MAX: ::std::os::raw::c_uint = 40;
pub const _SC_EQUIV_CLASS_MAX: ::std::os::raw::c_uint = 41;
pub const _SC_EXPR_NEST_MAX: ::std::os::raw::c_uint = 42;
pub const _SC_LINE_MAX: ::std::os::raw::c_uint = 43;
pub const _SC_RE_DUP_MAX: ::std::os::raw::c_uint = 44;
pub const _SC_CHARCLASS_NAME_MAX: ::std::os::raw::c_uint = 45;
pub const _SC_2_VERSION: ::std::os::raw::c_uint = 46;
pub const _SC_2_C_BIND: ::std::os::raw::c_uint = 47;
pub const _SC_2_C_DEV: ::std::os::raw::c_uint = 48;
pub const _SC_2_FORT_DEV: ::std::os::raw::c_uint = 49;
pub const _SC_2_FORT_RUN: ::std::os::raw::c_uint = 50;
pub const _SC_2_SW_DEV: ::std::os::raw::c_uint = 51;
pub const _SC_2_LOCALEDEF: ::std::os::raw::c_uint = 52;
pub const _SC_PII: ::std::os::raw::c_uint = 53;
pub const _SC_PII_XTI: ::std::os::raw::c_uint = 54;
pub const _SC_PII_SOCKET: ::std::os::raw::c_uint = 55;
pub const _SC_PII_INTERNET: ::std::os::raw::c_uint = 56;
pub const _SC_PII_OSI: ::std::os::raw::c_uint = 57;
pub const _SC_POLL: ::std::os::raw::c_uint = 58;
pub const _SC_SELECT: ::std::os::raw::c_uint = 59;
pub const _SC_UIO_MAXIOV: ::std::os::raw::c_uint = 60;
pub const _SC_IOV_MAX: ::std::os::raw::c_uint = 60;
pub const _SC_PII_INTERNET_STREAM: ::std::os::raw::c_uint = 61;
pub const _SC_PII_INTERNET_DGRAM: ::std::os::raw::c_uint = 62;
pub const _SC_PII_OSI_COTS: ::std::os::raw::c_uint = 63;
pub const _SC_PII_OSI_CLTS: ::std::os::raw::c_uint = 64;
pub const _SC_PII_OSI_M: ::std::os::raw::c_uint = 65;
pub const _SC_T_IOV_MAX: ::std::os::raw::c_uint = 66;
pub const _SC_THREADS: ::std::os::raw::c_uint = 67;
pub const _SC_THREAD_SAFE_FUNCTIONS: ::std::os::raw::c_uint = 68;
pub const _SC_GETGR_R_SIZE_MAX: ::std::os::raw::c_uint = 69;
pub const _SC_GETPW_R_SIZE_MAX: ::std::os::raw::c_uint = 70;
pub const _SC_LOGIN_NAME_MAX: ::std::os::raw::c_uint = 71;
pub const _SC_TTY_NAME_MAX: ::std::os::raw::c_uint = 72;
pub const _SC_THREAD_DESTRUCTOR_ITERATIONS: ::std::os::raw::c_uint = 73;
pub const _SC_THREAD_KEYS_MAX: ::std::os::raw::c_uint = 74;
pub const _SC_THREAD_STACK_MIN: ::std::os::raw::c_uint = 75;
pub const _SC_THREAD_THREADS_MAX: ::std::os::raw::c_uint = 76;
pub const _SC_THREAD_ATTR_STACKADDR: ::std::os::raw::c_uint = 77;
pub const _SC_THREAD_ATTR_STACKSIZE: ::std::os::raw::c_uint = 78;
pub const _SC_THREAD_PRIORITY_SCHEDULING: ::std::os::raw::c_uint = 79;
pub const _SC_THREAD_PRIO_INHERIT: ::std::os::raw::c_uint = 80;
pub const _SC_THREAD_PRIO_PROTECT: ::std::os::raw::c_uint = 81;
pub const _SC_THREAD_PROCESS_SHARED: ::std::os::raw::c_uint = 82;
pub const _SC_NPROCESSORS_CONF: ::std::os::raw::c_uint = 83;
pub const _SC_NPROCESSORS_ONLN: ::std::os::raw::c_uint = 84;
pub const _SC_PHYS_PAGES: ::std::os::raw::c_uint = 85;
pub const _SC_AVPHYS_PAGES: ::std::os::raw::c_uint = 86;
pub const _SC_ATEXIT_MAX: ::std::os::raw::c_uint = 87;
pub const _SC_PASS_MAX: ::std::os::raw::c_uint = 88;
pub const _SC_XOPEN_VERSION: ::std::os::raw::c_uint = 89;
pub const _SC_XOPEN_XCU_VERSION: ::std::os::raw::c_uint = 90;
pub const _SC_XOPEN_UNIX: ::std::os::raw::c_uint = 91;
pub const _SC_XOPEN_CRYPT: ::std::os::raw::c_uint = 92;
pub const _SC_XOPEN_ENH_I18N: ::std::os::raw::c_uint = 93;
pub const _SC_XOPEN_SHM: ::std::os::raw::c_uint = 94;
pub const _SC_2_CHAR_TERM: ::std::os::raw::c_uint = 95;
pub const _SC_2_C_VERSION: ::std::os::raw::c_uint = 96;
pub const _SC_2_UPE: ::std::os::raw::c_uint = 97;
pub const _SC_XOPEN_XPG2: ::std::os::raw::c_uint = 98;
pub const _SC_XOPEN_XPG3: ::std::os::raw::c_uint = 99;
pub const _SC_XOPEN_XPG4: ::std::os::raw::c_uint = 100;
pub const _SC_CHAR_BIT: ::std::os::raw::c_uint = 101;
pub const _SC_CHAR_MAX: ::std::os::raw::c_uint = 102;
pub const _SC_CHAR_MIN: ::std::os::raw::c_uint = 103;
pub const _SC_INT_MAX: ::std::os::raw::c_uint = 104;
pub const _SC_INT_MIN: ::std::os::raw::c_uint = 105;
pub const _SC_LONG_BIT: ::std::os::raw::c_uint = 106;
pub const _SC_WORD_BIT: ::std::os::raw::c_uint = 107;
pub const _SC_MB_LEN_MAX: ::std::os::raw::c_uint = 108;
pub const _SC_NZERO: ::std::os::raw::c_uint = 109;
pub const _SC_SSIZE_MAX: ::std::os::raw::c_uint = 110;
pub const _SC_SCHAR_MAX: ::std::os::raw::c_uint = 111;
pub const _SC_SCHAR_MIN: ::std::os::raw::c_uint = 112;
pub const _SC_SHRT_MAX: ::std::os::raw::c_uint = 113;
pub const _SC_SHRT_MIN: ::std::os::raw::c_uint = 114;
pub const _SC_UCHAR_MAX: ::std::os::raw::c_uint = 115;
pub const _SC_UINT_MAX: ::std::os::raw::c_uint = 116;
pub const _SC_ULONG_MAX: ::std::os::raw::c_uint = 117;
pub const _SC_USHRT_MAX: ::std::os::raw::c_uint = 118;
pub const _SC_NL_ARGMAX: ::std::os::raw::c_uint = 119;
pub const _SC_NL_LANGMAX: ::std::os::raw::c_uint = 120;
pub const _SC_NL_MSGMAX: ::std::os::raw::c_uint = 121;
pub const _SC_NL_NMAX: ::std::os::raw::c_uint = 122;
pub const _SC_NL_SETMAX: ::std::os::raw::c_uint = 123;
pub const _SC_NL_TEXTMAX: ::std::os::raw::c_uint = 124;
pub const _SC_XBS5_ILP32_OFF32: ::std::os::raw::c_uint = 125;
pub const _SC_XBS5_ILP32_OFFBIG: ::std::os::raw::c_uint = 126;
pub const _SC_XBS5_LP64_OFF64: ::std::os::raw::c_uint = 127;
pub const _SC_XBS5_LPBIG_OFFBIG: ::std::os::raw::c_uint = 128;
pub const _SC_XOPEN_LEGACY: ::std::os::raw::c_uint = 129;
pub const _SC_XOPEN_REALTIME: ::std::os::raw::c_uint = 130;
pub const _SC_XOPEN_REALTIME_THREADS: ::std::os::raw::c_uint = 131;
pub const _SC_ADVISORY_INFO: ::std::os::raw::c_uint = 132;
pub const _SC_BARRIERS: ::std::os::raw::c_uint = 133;
pub const _SC_BASE: ::std::os::raw::c_uint = 134;
pub const _SC_C_LANG_SUPPORT: ::std::os::raw::c_uint = 135;
pub const _SC_C_LANG_SUPPORT_R: ::std::os::raw::c_uint = 136;
pub const _SC_CLOCK_SELECTION: ::std::os::raw::c_uint = 137;
pub const _SC_CPUTIME: ::std::os::raw::c_uint = 138;
pub const _SC_THREAD_CPUTIME: ::std::os::raw::c_uint = 139;
pub const _SC_DEVICE_IO: ::std::os::raw::c_uint = 140;
pub const _SC_DEVICE_SPECIFIC: ::std::os::raw::c_uint = 141;
pub const _SC_DEVICE_SPECIFIC_R: ::std::os::raw::c_uint = 142;
pub const _SC_FD_MGMT: ::std::os::raw::c_uint = 143;
pub const _SC_FIFO: ::std::os::raw::c_uint = 144;
pub const _SC_PIPE: ::std::os::raw::c_uint = 145;
pub const _SC_FILE_ATTRIBUTES: ::std::os::raw::c_uint = 146;
pub const _SC_FILE_LOCKING: ::std::os::raw::c_uint = 147;
pub const _SC_FILE_SYSTEM: ::std::os::raw::c_uint = 148;
pub const _SC_MONOTONIC_CLOCK: ::std::os::raw::c_uint = 149;
pub const _SC_MULTI_PROCESS: ::std::os::raw::c_uint = 150;
pub const _SC_SINGLE_PROCESS: ::std::os::raw::c_uint = 151;
pub const _SC_NETWORKING: ::std::os::raw::c_uint = 152;
pub const _SC_READER_WRITER_LOCKS: ::std::os::raw::c_uint = 153;
pub const _SC_SPIN_LOCKS: ::std::os::raw::c_uint = 154;
pub const _SC_REGEXP: ::std::os::raw::c_uint = 155;
pub const _SC_REGEX_VERSION: ::std::os::raw::c_uint = 156;
pub const _SC_SHELL: ::std::os::raw::c_uint = 157;
pub const _SC_SIGNALS: ::std::os::raw::c_uint = 158;
pub const _SC_SPAWN: ::std::os::raw::c_uint = 159;
pub const _SC_SPORADIC_SERVER: ::std::os::raw::c_uint = 160;
pub const _SC_THREAD_SPORADIC_SERVER: ::std::os::raw::c_uint = 161;
pub const _SC_SYSTEM_DATABASE: ::std::os::raw::c_uint = 162;
pub const _SC_SYSTEM_DATABASE_R: ::std::os::raw::c_uint = 163;
pub const _SC_TIMEOUTS: ::std::os::raw::c_uint = 164;
pub const _SC_TYPED_MEMORY_OBJECTS: ::std::os::raw::c_uint = 165;
pub const _SC_USER_GROUPS: ::std::os::raw::c_uint = 166;
pub const _SC_USER_GROUPS_R: ::std::os::raw::c_uint = 167;
pub const _SC_2_PBS: ::std::os::raw::c_uint = 168;
pub const _SC_2_PBS_ACCOUNTING: ::std::os::raw::c_uint = 169;
pub const _SC_2_PBS_LOCATE: ::std::os::raw::c_uint = 170;
pub const _SC_2_PBS_MESSAGE: ::std::os::raw::c_uint = 171;
pub const _SC_2_PBS_TRACK: ::std::os::raw::c_uint = 172;
pub const _SC_SYMLOOP_MAX: ::std::os::raw::c_uint = 173;
pub const _SC_STREAMS: ::std::os::raw::c_uint = 174;
pub const _SC_2_PBS_CHECKPOINT: ::std::os::raw::c_uint = 175;
pub const _SC_V6_ILP32_OFF32: ::std::os::raw::c_uint = 176;
pub const _SC_V6_ILP32_OFFBIG: ::std::os::raw::c_uint = 177;
pub const _SC_V6_LP64_OFF64: ::std::os::raw::c_uint = 178;
pub const _SC_V6_LPBIG_OFFBIG: ::std::os::raw::c_uint = 179;
pub const _SC_HOST_NAME_MAX: ::std::os::raw::c_uint = 180;
pub const _SC_TRACE: ::std::os::raw::c_uint = 181;
pub const _SC_TRACE_EVENT_FILTER: ::std::os::raw::c_uint = 182;
pub const _SC_TRACE_INHERIT: ::std::os::raw::c_uint = 183;
pub const _SC_TRACE_LOG: ::std::os::raw::c_uint = 184;
pub const _SC_LEVEL1_ICACHE_SIZE: ::std::os::raw::c_uint = 185;
pub const _SC_LEVEL1_ICACHE_ASSOC: ::std::os::raw::c_uint = 186;
pub const _SC_LEVEL1_ICACHE_LINESIZE: ::std::os::raw::c_uint = 187;
pub const _SC_LEVEL1_DCACHE_SIZE: ::std::os::raw::c_uint = 188;
pub const _SC_LEVEL1_DCACHE_ASSOC: ::std::os::raw::c_uint = 189;
pub const _SC_LEVEL1_DCACHE_LINESIZE: ::std::os::raw::c_uint = 190;
pub const _SC_LEVEL2_CACHE_SIZE: ::std::os::raw::c_uint = 191;
pub const _SC_LEVEL2_CACHE_ASSOC: ::std::os::raw::c_uint = 192;
pub const _SC_LEVEL2_CACHE_LINESIZE: ::std::os::raw::c_uint = 193;
pub const _SC_LEVEL3_CACHE_SIZE: ::std::os::raw::c_uint = 194;
pub const _SC_LEVEL3_CACHE_ASSOC: ::std::os::raw::c_uint = 195;
pub const _SC_LEVEL3_CACHE_LINESIZE: ::std::os::raw::c_uint = 196;
pub const _SC_LEVEL4_CACHE_SIZE: ::std::os::raw::c_uint = 197;
pub const _SC_LEVEL4_CACHE_ASSOC: ::std::os::raw::c_uint = 198;
pub const _SC_LEVEL4_CACHE_LINESIZE: ::std::os::raw::c_uint = 199;
pub const _SC_IPV6: ::std::os::raw::c_uint = 235;
pub const _SC_RAW_SOCKETS: ::std::os::raw::c_uint = 236;
pub const _SC_V7_ILP32_OFF32: ::std::os::raw::c_uint = 237;
pub const _SC_V7_ILP32_OFFBIG: ::std::os::raw::c_uint = 238;
pub const _SC_V7_LP64_OFF64: ::std::os::raw::c_uint = 239;
pub const _SC_V7_LPBIG_OFFBIG: ::std::os::raw::c_uint = 240;
pub const _SC_SS_REPL_MAX: ::std::os::raw::c_uint = 241;
pub const _SC_TRACE_EVENT_NAME_MAX: ::std::os::raw::c_uint = 242;
pub const _SC_TRACE_NAME_MAX: ::std::os::raw::c_uint = 243;
pub const _SC_TRACE_SYS_MAX: ::std::os::raw::c_uint = 244;
pub const _SC_TRACE_USER_EVENT_MAX: ::std::os::raw::c_uint = 245;
pub const _SC_XOPEN_STREAMS: ::std::os::raw::c_uint = 246;
pub const _SC_THREAD_ROBUST_PRIO_INHERIT: ::std::os::raw::c_uint = 247;
pub const _SC_THREAD_ROBUST_PRIO_PROTECT: ::std::os::raw::c_uint = 248;
pub type _bindgen_ty_2 = ::std::os::raw::c_uint;
pub const _CS_PATH: ::std::os::raw::c_uint = 0;
pub const _CS_V6_WIDTH_RESTRICTED_ENVS: ::std::os::raw::c_uint = 1;
pub const _CS_GNU_LIBC_VERSION: ::std::os::raw::c_uint = 2;
pub const _CS_GNU_LIBPTHREAD_VERSION: ::std::os::raw::c_uint = 3;
pub const _CS_V5_WIDTH_RESTRICTED_ENVS: ::std::os::raw::c_uint = 4;
pub const _CS_V7_WIDTH_RESTRICTED_ENVS: ::std::os::raw::c_uint = 5;
pub const _CS_LFS_CFLAGS: ::std::os::raw::c_uint = 1000;
pub const _CS_LFS_LDFLAGS: ::std::os::raw::c_uint = 1001;
pub const _CS_LFS_LIBS: ::std::os::raw::c_uint = 1002;
pub const _CS_LFS_LINTFLAGS: ::std::os::raw::c_uint = 1003;
pub const _CS_LFS64_CFLAGS: ::std::os::raw::c_uint = 1004;
pub const _CS_LFS64_LDFLAGS: ::std::os::raw::c_uint = 1005;
pub const _CS_LFS64_LIBS: ::std::os::raw::c_uint = 1006;
pub const _CS_LFS64_LINTFLAGS: ::std::os::raw::c_uint = 1007;
pub const _CS_XBS5_ILP32_OFF32_CFLAGS: ::std::os::raw::c_uint = 1100;
pub const _CS_XBS5_ILP32_OFF32_LDFLAGS: ::std::os::raw::c_uint = 1101;
pub const _CS_XBS5_ILP32_OFF32_LIBS: ::std::os::raw::c_uint = 1102;
pub const _CS_XBS5_ILP32_OFF32_LINTFLAGS: ::std::os::raw::c_uint = 1103;
pub const _CS_XBS5_ILP32_OFFBIG_CFLAGS: ::std::os::raw::c_uint = 1104;
pub const _CS_XBS5_ILP32_OFFBIG_LDFLAGS: ::std::os::raw::c_uint = 1105;
pub const _CS_XBS5_ILP32_OFFBIG_LIBS: ::std::os::raw::c_uint = 1106;
pub const _CS_XBS5_ILP32_OFFBIG_LINTFLAGS: ::std::os::raw::c_uint = 1107;
pub const _CS_XBS5_LP64_OFF64_CFLAGS: ::std::os::raw::c_uint = 1108;
pub const _CS_XBS5_LP64_OFF64_LDFLAGS: ::std::os::raw::c_uint = 1109;
pub const _CS_XBS5_LP64_OFF64_LIBS: ::std::os::raw::c_uint = 1110;
pub const _CS_XBS5_LP64_OFF64_LINTFLAGS: ::std::os::raw::c_uint = 1111;
pub const _CS_XBS5_LPBIG_OFFBIG_CFLAGS: ::std::os::raw::c_uint = 1112;
pub const _CS_XBS5_LPBIG_OFFBIG_LDFLAGS: ::std::os::raw::c_uint = 1113;
pub const _CS_XBS5_LPBIG_OFFBIG_LIBS: ::std::os::raw::c_uint = 1114;
pub const _CS_XBS5_LPBIG_OFFBIG_LINTFLAGS: ::std::os::raw::c_uint = 1115;
pub const _CS_POSIX_V6_ILP32_OFF32_CFLAGS: ::std::os::raw::c_uint = 1116;
pub const _CS_POSIX_V6_ILP32_OFF32_LDFLAGS: ::std::os::raw::c_uint = 1117;
pub const _CS_POSIX_V6_ILP32_OFF32_LIBS: ::std::os::raw::c_uint = 1118;
pub const _CS_POSIX_V6_ILP32_OFF32_LINTFLAGS: ::std::os::raw::c_uint = 1119;
pub const _CS_POSIX_V6_ILP32_OFFBIG_CFLAGS: ::std::os::raw::c_uint = 1120;
pub const _CS_POSIX_V6_ILP32_OFFBIG_LDFLAGS: ::std::os::raw::c_uint = 1121;
pub const _CS_POSIX_V6_ILP32_OFFBIG_LIBS: ::std::os::raw::c_uint = 1122;
pub const _CS_POSIX_V6_ILP32_OFFBIG_LINTFLAGS: ::std::os::raw::c_uint = 1123;
pub const _CS_POSIX_V6_LP64_OFF64_CFLAGS: ::std::os::raw::c_uint = 1124;
pub const _CS_POSIX_V6_LP64_OFF64_LDFLAGS: ::std::os::raw::c_uint = 1125;
pub const _CS_POSIX_V6_LP64_OFF64_LIBS: ::std::os::raw::c_uint = 1126;
pub const _CS_POSIX_V6_LP64_OFF64_LINTFLAGS: ::std::os::raw::c_uint = 1127;
pub const _CS_POSIX_V6_LPBIG_OFFBIG_CFLAGS: ::std::os::raw::c_uint = 1128;
pub const _CS_POSIX_V6_LPBIG_OFFBIG_LDFLAGS: ::std::os::raw::c_uint = 1129;
pub const _CS_POSIX_V6_LPBIG_OFFBIG_LIBS: ::std::os::raw::c_uint = 1130;
pub const _CS_POSIX_V6_LPBIG_OFFBIG_LINTFLAGS: ::std::os::raw::c_uint = 1131;
pub const _CS_POSIX_V7_ILP32_OFF32_CFLAGS: ::std::os::raw::c_uint = 1132;
pub const _CS_POSIX_V7_ILP32_OFF32_LDFLAGS: ::std::os::raw::c_uint = 1133;
pub const _CS_POSIX_V7_ILP32_OFF32_LIBS: ::std::os::raw::c_uint = 1134;
pub const _CS_POSIX_V7_ILP32_OFF32_LINTFLAGS: ::std::os::raw::c_uint = 1135;
pub const _CS_POSIX_V7_ILP32_OFFBIG_CFLAGS: ::std::os::raw::c_uint = 1136;
pub const _CS_POSIX_V7_ILP32_OFFBIG_LDFLAGS: ::std::os::raw::c_uint = 1137;
pub const _CS_POSIX_V7_ILP32_OFFBIG_LIBS: ::std::os::raw::c_uint = 1138;
pub const _CS_POSIX_V7_ILP32_OFFBIG_LINTFLAGS: ::std::os::raw::c_uint = 1139;
pub const _CS_POSIX_V7_LP64_OFF64_CFLAGS: ::std::os::raw::c_uint = 1140;
pub const _CS_POSIX_V7_LP64_OFF64_LDFLAGS: ::std::os::raw::c_uint = 1141;
pub const _CS_POSIX_V7_LP64_OFF64_LIBS: ::std::os::raw::c_uint = 1142;
pub const _CS_POSIX_V7_LP64_OFF64_LINTFLAGS: ::std::os::raw::c_uint = 1143;
pub const _CS_POSIX_V7_LPBIG_OFFBIG_CFLAGS: ::std::os::raw::c_uint = 1144;
pub const _CS_POSIX_V7_LPBIG_OFFBIG_LDFLAGS: ::std::os::raw::c_uint = 1145;
pub const _CS_POSIX_V7_LPBIG_OFFBIG_LIBS: ::std::os::raw::c_uint = 1146;
pub const _CS_POSIX_V7_LPBIG_OFFBIG_LINTFLAGS: ::std::os::raw::c_uint = 1147;
pub const _CS_V6_ENV: ::std::os::raw::c_uint = 1148;
pub const _CS_V7_ENV: ::std::os::raw::c_uint = 1149;
pub type _bindgen_ty_3 = ::std::os::raw::c_uint;
extern "C" {
    pub fn pathconf(
        __path: *const ::std::os::raw::c_char,
        __name: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_long;
}
extern "C" {
    pub fn fpathconf(
        __fd: ::std::os::raw::c_int,
        __name: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_long;
}
extern "C" {
    pub fn sysconf(__name: ::std::os::raw::c_int) -> ::std::os::raw::c_long;
}
extern "C" {
    pub fn confstr(
        __name: ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_char,
        __len: size_t,
    ) -> size_t;
}
extern "C" {
    pub fn getpid() -> __pid_t;
}
extern "C" {
    pub fn getppid() -> __pid_t;
}
extern "C" {
    pub fn getpgrp() -> __pid_t;
}
extern "C" {
    pub fn __getpgid(__pid: __pid_t) -> __pid_t;
}
extern "C" {
    pub fn getpgid(__pid: __pid_t) -> __pid_t;
}
extern "C" {
    pub fn setpgid(__pid: __pid_t, __pgid: __pid_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn setpgrp() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn setsid() -> __pid_t;
}
extern "C" {
    pub fn getsid(__pid: __pid_t) -> __pid_t;
}
extern "C" {
    pub fn getuid() -> __uid_t;
}
extern "C" {
    pub fn geteuid() -> __uid_t;
}
extern "C" {
    pub fn getgid() -> __gid_t;
}
extern "C" {
    pub fn getegid() -> __gid_t;
}
extern "C" {
    pub fn getgroups(__size: ::std::os::raw::c_int, __list: *mut __gid_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn group_member(__gid: __gid_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn setuid(__uid: __uid_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn setreuid(__ruid: __uid_t, __euid: __uid_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn seteuid(__uid: __uid_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn setgid(__gid: __gid_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn setregid(__rgid: __gid_t, __egid: __gid_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn setegid(__gid: __gid_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn getresuid(
        __ruid: *mut __uid_t,
        __euid: *mut __uid_t,
        __suid: *mut __uid_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn getresgid(
        __rgid: *mut __gid_t,
        __egid: *mut __gid_t,
        __sgid: *mut __gid_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn setresuid(__ruid: __uid_t, __euid: __uid_t, __suid: __uid_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn setresgid(__rgid: __gid_t, __egid: __gid_t, __sgid: __gid_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn fork() -> __pid_t;
}
extern "C" {
    pub fn vfork() -> __pid_t;
}
extern "C" {
    pub fn ttyname(__fd: ::std::os::raw::c_int) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn ttyname_r(
        __fd: ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_char,
        __buflen: size_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn isatty(__fd: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn ttyslot() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn link(
        __from: *const ::std::os::raw::c_char,
        __to: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn linkat(
        __fromfd: ::std::os::raw::c_int,
        __from: *const ::std::os::raw::c_char,
        __tofd: ::std::os::raw::c_int,
        __to: *const ::std::os::raw::c_char,
        __flags: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn symlink(
        __from: *const ::std::os::raw::c_char,
        __to: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn readlink(
        __path: *const ::std::os::raw::c_char,
        __buf: *mut ::std::os::raw::c_char,
        __len: size_t,
    ) -> ssize_t;
}
extern "C" {
    pub fn symlinkat(
        __from: *const ::std::os::raw::c_char,
        __tofd: ::std::os::raw::c_int,
        __to: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn readlinkat(
        __fd: ::std::os::raw::c_int,
        __path: *const ::std::os::raw::c_char,
        __buf: *mut ::std::os::raw::c_char,
        __len: size_t,
    ) -> ssize_t;
}
extern "C" {
    pub fn unlink(__name: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn unlinkat(
        __fd: ::std::os::raw::c_int,
        __name: *const ::std::os::raw::c_char,
        __flag: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn rmdir(__path: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn tcgetpgrp(__fd: ::std::os::raw::c_int) -> __pid_t;
}
extern "C" {
    pub fn tcsetpgrp(__fd: ::std::os::raw::c_int, __pgrp_id: __pid_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn getlogin() -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn getlogin_r(
        __name: *mut ::std::os::raw::c_char,
        __name_len: size_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn setlogin(__name: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
    pub static mut optarg: *mut ::std::os::raw::c_char;
}
extern "C" {
    pub static mut optind: ::std::os::raw::c_int;
}
extern "C" {
    pub static mut opterr: ::std::os::raw::c_int;
}
extern "C" {
    pub static mut optopt: ::std::os::raw::c_int;
}
extern "C" {
    pub fn getopt(
        ___argc: ::std::os::raw::c_int,
        ___argv: *const *mut ::std::os::raw::c_char,
        __shortopts: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn gethostname(__name: *mut ::std::os::raw::c_char, __len: size_t)
        -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn sethostname(
        __name: *const ::std::os::raw::c_char,
        __len: size_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn sethostid(__id: ::std::os::raw::c_long) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn getdomainname(
        __name: *mut ::std::os::raw::c_char,
        __len: size_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn setdomainname(
        __name: *const ::std::os::raw::c_char,
        __len: size_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn vhangup() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn revoke(__file: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn profil(
        __sample_buffer: *mut ::std::os::raw::c_ushort,
        __size: size_t,
        __offset: size_t,
        __scale: ::std::os::raw::c_uint,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn acct(__name: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn getusershell() -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn endusershell();
}
extern "C" {
    pub fn setusershell();
}
extern "C" {
    pub fn daemon(
        __nochdir: ::std::os::raw::c_int,
        __noclose: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn chroot(__path: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn getpass(__prompt: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn fsync(__fd: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn syncfs(__fd: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn gethostid() -> ::std::os::raw::c_long;
}
extern "C" {
    pub fn sync();
}
extern "C" {
    pub fn getpagesize() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn getdtablesize() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn truncate(
        __file: *const ::std::os::raw::c_char,
        __length: __off_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn truncate64(
        __file: *const ::std::os::raw::c_char,
        __length: __off64_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn ftruncate(__fd: ::std::os::raw::c_int, __length: __off_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn ftruncate64(__fd: ::std::os::raw::c_int, __length: __off64_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn brk(__addr: *mut ::std::os::raw::c_void) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn sbrk(__delta: isize) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn syscall(__sysno: ::std::os::raw::c_long, ...) -> ::std::os::raw::c_long;
}
extern "C" {
    pub fn lockf(
        __fd: ::std::os::raw::c_int,
        __cmd: ::std::os::raw::c_int,
        __len: __off_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn lockf64(
        __fd: ::std::os::raw::c_int,
        __cmd: ::std::os::raw::c_int,
        __len: __off64_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn copy_file_range(
        __infd: ::std::os::raw::c_int,
        __pinoff: *mut __off64_t,
        __outfd: ::std::os::raw::c_int,
        __poutoff: *mut __off64_t,
        __length: size_t,
        __flags: ::std::os::raw::c_uint,
    ) -> ssize_t;
}
extern "C" {
    pub fn fdatasync(__fildes: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn crypt(
        __key: *const ::std::os::raw::c_char,
        __salt: *const ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn encrypt(__glibc_block: *mut ::std::os::raw::c_char, __edflag: ::std::os::raw::c_int);
}
extern "C" {
    pub fn swab(
        __from: *const ::std::os::raw::c_void,
        __to: *mut ::std::os::raw::c_void,
        __n: ssize_t,
    );
}
extern "C" {
    pub fn getentropy(
        __buffer: *mut ::std::os::raw::c_void,
        __length: size_t,
    ) -> ::std::os::raw::c_int;
}
pub type __int64 = ::std::os::raw::c_longlong;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct futureTricks {
    pub nodes: ::std::os::raw::c_int,
    pub cards: ::std::os::raw::c_int,
    pub suit: [::std::os::raw::c_int; 13usize],
    pub rank: [::std::os::raw::c_int; 13usize],
    pub equals: [::std::os::raw::c_int; 13usize],
    pub score: [::std::os::raw::c_int; 13usize],
}
#[test]
fn bindgen_test_layout_futureTricks() {
    assert_eq!(
        ::std::mem::size_of::<futureTricks>(),
        216usize,
        concat!("Size of: ", stringify!(futureTricks))
    );
    assert_eq!(
        ::std::mem::align_of::<futureTricks>(),
        4usize,
        concat!("Alignment of ", stringify!(futureTricks))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<futureTricks>())).nodes as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(futureTricks),
            "::",
            stringify!(nodes)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<futureTricks>())).cards as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(futureTricks),
            "::",
            stringify!(cards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<futureTricks>())).suit as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(futureTricks),
            "::",
            stringify!(suit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<futureTricks>())).rank as *const _ as usize },
        60usize,
        concat!(
            "Offset of field: ",
            stringify!(futureTricks),
            "::",
            stringify!(rank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<futureTricks>())).equals as *const _ as usize },
        112usize,
        concat!(
            "Offset of field: ",
            stringify!(futureTricks),
            "::",
            stringify!(equals)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<futureTricks>())).score as *const _ as usize },
        164usize,
        concat!(
            "Offset of field: ",
            stringify!(futureTricks),
            "::",
            stringify!(score)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct deal {
    pub trump: ::std::os::raw::c_int,
    pub first: ::std::os::raw::c_int,
    pub currentTrickSuit: [::std::os::raw::c_int; 3usize],
    pub currentTrickRank: [::std::os::raw::c_int; 3usize],
    pub remainCards: [[::std::os::raw::c_uint; 4usize]; 4usize],
}
#[test]
fn bindgen_test_layout_deal() {
    assert_eq!(
        ::std::mem::size_of::<deal>(),
        96usize,
        concat!("Size of: ", stringify!(deal))
    );
    assert_eq!(
        ::std::mem::align_of::<deal>(),
        4usize,
        concat!("Alignment of ", stringify!(deal))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<deal>())).trump as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(deal),
            "::",
            stringify!(trump)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<deal>())).first as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(deal),
            "::",
            stringify!(first)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<deal>())).currentTrickSuit as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(deal),
            "::",
            stringify!(currentTrickSuit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<deal>())).currentTrickRank as *const _ as usize },
        20usize,
        concat!(
            "Offset of field: ",
            stringify!(deal),
            "::",
            stringify!(currentTrickRank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<deal>())).remainCards as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(deal),
            "::",
            stringify!(remainCards)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct dealPBN {
    pub trump: ::std::os::raw::c_int,
    pub first: ::std::os::raw::c_int,
    pub currentTrickSuit: [::std::os::raw::c_int; 3usize],
    pub currentTrickRank: [::std::os::raw::c_int; 3usize],
    pub remainCards: [::std::os::raw::c_char; 80usize],
}
#[test]
fn bindgen_test_layout_dealPBN() {
    assert_eq!(
        ::std::mem::size_of::<dealPBN>(),
        112usize,
        concat!("Size of: ", stringify!(dealPBN))
    );
    assert_eq!(
        ::std::mem::align_of::<dealPBN>(),
        4usize,
        concat!("Alignment of ", stringify!(dealPBN))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<dealPBN>())).trump as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(dealPBN),
            "::",
            stringify!(trump)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<dealPBN>())).first as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(dealPBN),
            "::",
            stringify!(first)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<dealPBN>())).currentTrickSuit as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(dealPBN),
            "::",
            stringify!(currentTrickSuit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<dealPBN>())).currentTrickRank as *const _ as usize },
        20usize,
        concat!(
            "Offset of field: ",
            stringify!(dealPBN),
            "::",
            stringify!(currentTrickRank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<dealPBN>())).remainCards as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(dealPBN),
            "::",
            stringify!(remainCards)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct boards {
    pub noOfBoards: ::std::os::raw::c_int,
    pub deals: [deal; 200usize],
    pub target: [::std::os::raw::c_int; 200usize],
    pub solutions: [::std::os::raw::c_int; 200usize],
    pub mode: [::std::os::raw::c_int; 200usize],
}
#[test]
fn bindgen_test_layout_boards() {
    assert_eq!(
        ::std::mem::size_of::<boards>(),
        21604usize,
        concat!("Size of: ", stringify!(boards))
    );
    assert_eq!(
        ::std::mem::align_of::<boards>(),
        4usize,
        concat!("Alignment of ", stringify!(boards))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<boards>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(boards),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<boards>())).deals as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(boards),
            "::",
            stringify!(deals)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<boards>())).target as *const _ as usize },
        19204usize,
        concat!(
            "Offset of field: ",
            stringify!(boards),
            "::",
            stringify!(target)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<boards>())).solutions as *const _ as usize },
        20004usize,
        concat!(
            "Offset of field: ",
            stringify!(boards),
            "::",
            stringify!(solutions)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<boards>())).mode as *const _ as usize },
        20804usize,
        concat!(
            "Offset of field: ",
            stringify!(boards),
            "::",
            stringify!(mode)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct boardsPBN {
    pub noOfBoards: ::std::os::raw::c_int,
    pub deals: [dealPBN; 200usize],
    pub target: [::std::os::raw::c_int; 200usize],
    pub solutions: [::std::os::raw::c_int; 200usize],
    pub mode: [::std::os::raw::c_int; 200usize],
}
#[test]
fn bindgen_test_layout_boardsPBN() {
    assert_eq!(
        ::std::mem::size_of::<boardsPBN>(),
        24804usize,
        concat!("Size of: ", stringify!(boardsPBN))
    );
    assert_eq!(
        ::std::mem::align_of::<boardsPBN>(),
        4usize,
        concat!("Alignment of ", stringify!(boardsPBN))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<boardsPBN>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(boardsPBN),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<boardsPBN>())).deals as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(boardsPBN),
            "::",
            stringify!(deals)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<boardsPBN>())).target as *const _ as usize },
        22404usize,
        concat!(
            "Offset of field: ",
            stringify!(boardsPBN),
            "::",
            stringify!(target)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<boardsPBN>())).solutions as *const _ as usize },
        23204usize,
        concat!(
            "Offset of field: ",
            stringify!(boardsPBN),
            "::",
            stringify!(solutions)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<boardsPBN>())).mode as *const _ as usize },
        24004usize,
        concat!(
            "Offset of field: ",
            stringify!(boardsPBN),
            "::",
            stringify!(mode)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct solvedBoards {
    pub noOfBoards: ::std::os::raw::c_int,
    pub solvedBoard: [futureTricks; 200usize],
}
#[test]
fn bindgen_test_layout_solvedBoards() {
    assert_eq!(
        ::std::mem::size_of::<solvedBoards>(),
        43204usize,
        concat!("Size of: ", stringify!(solvedBoards))
    );
    assert_eq!(
        ::std::mem::align_of::<solvedBoards>(),
        4usize,
        concat!("Alignment of ", stringify!(solvedBoards))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<solvedBoards>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedBoards),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<solvedBoards>())).solvedBoard as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedBoards),
            "::",
            stringify!(solvedBoard)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ddTableDeal {
    pub cards: [[::std::os::raw::c_uint; 4usize]; 4usize],
}
#[test]
fn bindgen_test_layout_ddTableDeal() {
    assert_eq!(
        ::std::mem::size_of::<ddTableDeal>(),
        64usize,
        concat!("Size of: ", stringify!(ddTableDeal))
    );
    assert_eq!(
        ::std::mem::align_of::<ddTableDeal>(),
        4usize,
        concat!("Alignment of ", stringify!(ddTableDeal))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableDeal>())).cards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableDeal),
            "::",
            stringify!(cards)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ddTableDeals {
    pub noOfTables: ::std::os::raw::c_int,
    pub deals: [ddTableDeal; 200usize],
}
#[test]
fn bindgen_test_layout_ddTableDeals() {
    assert_eq!(
        ::std::mem::size_of::<ddTableDeals>(),
        12804usize,
        concat!("Size of: ", stringify!(ddTableDeals))
    );
    assert_eq!(
        ::std::mem::align_of::<ddTableDeals>(),
        4usize,
        concat!("Alignment of ", stringify!(ddTableDeals))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableDeals>())).noOfTables as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableDeals),
            "::",
            stringify!(noOfTables)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableDeals>())).deals as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableDeals),
            "::",
            stringify!(deals)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ddTableDealPBN {
    pub cards: [::std::os::raw::c_char; 80usize],
}
#[test]
fn bindgen_test_layout_ddTableDealPBN() {
    assert_eq!(
        ::std::mem::size_of::<ddTableDealPBN>(),
        80usize,
        concat!("Size of: ", stringify!(ddTableDealPBN))
    );
    assert_eq!(
        ::std::mem::align_of::<ddTableDealPBN>(),
        1usize,
        concat!("Alignment of ", stringify!(ddTableDealPBN))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableDealPBN>())).cards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableDealPBN),
            "::",
            stringify!(cards)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ddTableDealsPBN {
    pub noOfTables: ::std::os::raw::c_int,
    pub deals: [ddTableDealPBN; 200usize],
}
#[test]
fn bindgen_test_layout_ddTableDealsPBN() {
    assert_eq!(
        ::std::mem::size_of::<ddTableDealsPBN>(),
        16004usize,
        concat!("Size of: ", stringify!(ddTableDealsPBN))
    );
    assert_eq!(
        ::std::mem::align_of::<ddTableDealsPBN>(),
        4usize,
        concat!("Alignment of ", stringify!(ddTableDealsPBN))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableDealsPBN>())).noOfTables as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableDealsPBN),
            "::",
            stringify!(noOfTables)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableDealsPBN>())).deals as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableDealsPBN),
            "::",
            stringify!(deals)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ddTableResults {
    pub resTable: [[::std::os::raw::c_int; 4usize]; 5usize],
}
#[test]
fn bindgen_test_layout_ddTableResults() {
    assert_eq!(
        ::std::mem::size_of::<ddTableResults>(),
        80usize,
        concat!("Size of: ", stringify!(ddTableResults))
    );
    assert_eq!(
        ::std::mem::align_of::<ddTableResults>(),
        4usize,
        concat!("Alignment of ", stringify!(ddTableResults))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableResults>())).resTable as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableResults),
            "::",
            stringify!(resTable)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ddTablesRes {
    pub noOfBoards: ::std::os::raw::c_int,
    pub results: [ddTableResults; 200usize],
}
#[test]
fn bindgen_test_layout_ddTablesRes() {
    assert_eq!(
        ::std::mem::size_of::<ddTablesRes>(),
        16004usize,
        concat!("Size of: ", stringify!(ddTablesRes))
    );
    assert_eq!(
        ::std::mem::align_of::<ddTablesRes>(),
        4usize,
        concat!("Alignment of ", stringify!(ddTablesRes))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTablesRes>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTablesRes),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTablesRes>())).results as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTablesRes),
            "::",
            stringify!(results)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct parResults {
    pub parScore: [[::std::os::raw::c_char; 16usize]; 2usize],
    pub parContractsString: [[::std::os::raw::c_char; 128usize]; 2usize],
}
#[test]
fn bindgen_test_layout_parResults() {
    assert_eq!(
        ::std::mem::size_of::<parResults>(),
        288usize,
        concat!("Size of: ", stringify!(parResults))
    );
    assert_eq!(
        ::std::mem::align_of::<parResults>(),
        1usize,
        concat!("Alignment of ", stringify!(parResults))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResults>())).parScore as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(parResults),
            "::",
            stringify!(parScore)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResults>())).parContractsString as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(parResults),
            "::",
            stringify!(parContractsString)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct allParResults {
    pub presults: [parResults; 40usize],
}
#[test]
fn bindgen_test_layout_allParResults() {
    assert_eq!(
        ::std::mem::size_of::<allParResults>(),
        11520usize,
        concat!("Size of: ", stringify!(allParResults))
    );
    assert_eq!(
        ::std::mem::align_of::<allParResults>(),
        1usize,
        concat!("Alignment of ", stringify!(allParResults))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<allParResults>())).presults as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(allParResults),
            "::",
            stringify!(presults)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct parResultsDealer {
    pub number: ::std::os::raw::c_int,
    pub score: ::std::os::raw::c_int,
    pub contracts: [[::std::os::raw::c_char; 10usize]; 10usize],
}
#[test]
fn bindgen_test_layout_parResultsDealer() {
    assert_eq!(
        ::std::mem::size_of::<parResultsDealer>(),
        108usize,
        concat!("Size of: ", stringify!(parResultsDealer))
    );
    assert_eq!(
        ::std::mem::align_of::<parResultsDealer>(),
        4usize,
        concat!("Alignment of ", stringify!(parResultsDealer))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResultsDealer>())).number as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(parResultsDealer),
            "::",
            stringify!(number)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResultsDealer>())).score as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(parResultsDealer),
            "::",
            stringify!(score)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResultsDealer>())).contracts as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(parResultsDealer),
            "::",
            stringify!(contracts)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct contractType {
    pub underTricks: ::std::os::raw::c_int,
    pub overTricks: ::std::os::raw::c_int,
    pub level: ::std::os::raw::c_int,
    pub denom: ::std::os::raw::c_int,
    pub seats: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_contractType() {
    assert_eq!(
        ::std::mem::size_of::<contractType>(),
        20usize,
        concat!("Size of: ", stringify!(contractType))
    );
    assert_eq!(
        ::std::mem::align_of::<contractType>(),
        4usize,
        concat!("Alignment of ", stringify!(contractType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<contractType>())).underTricks as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(contractType),
            "::",
            stringify!(underTricks)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<contractType>())).overTricks as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(contractType),
            "::",
            stringify!(overTricks)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<contractType>())).level as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(contractType),
            "::",
            stringify!(level)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<contractType>())).denom as *const _ as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(contractType),
            "::",
            stringify!(denom)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<contractType>())).seats as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(contractType),
            "::",
            stringify!(seats)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct parResultsMaster {
    pub score: ::std::os::raw::c_int,
    pub number: ::std::os::raw::c_int,
    pub contracts: [contractType; 10usize],
}
#[test]
fn bindgen_test_layout_parResultsMaster() {
    assert_eq!(
        ::std::mem::size_of::<parResultsMaster>(),
        208usize,
        concat!("Size of: ", stringify!(parResultsMaster))
    );
    assert_eq!(
        ::std::mem::align_of::<parResultsMaster>(),
        4usize,
        concat!("Alignment of ", stringify!(parResultsMaster))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResultsMaster>())).score as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(parResultsMaster),
            "::",
            stringify!(score)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResultsMaster>())).number as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(parResultsMaster),
            "::",
            stringify!(number)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResultsMaster>())).contracts as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(parResultsMaster),
            "::",
            stringify!(contracts)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct parTextResults {
    pub parText: [[::std::os::raw::c_char; 128usize]; 2usize],
    pub equal: bool,
}
#[test]
fn bindgen_test_layout_parTextResults() {
    assert_eq!(
        ::std::mem::size_of::<parTextResults>(),
        257usize,
        concat!("Size of: ", stringify!(parTextResults))
    );
    assert_eq!(
        ::std::mem::align_of::<parTextResults>(),
        1usize,
        concat!("Alignment of ", stringify!(parTextResults))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parTextResults>())).parText as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(parTextResults),
            "::",
            stringify!(parText)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parTextResults>())).equal as *const _ as usize },
        256usize,
        concat!(
            "Offset of field: ",
            stringify!(parTextResults),
            "::",
            stringify!(equal)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct playTraceBin {
    pub number: ::std::os::raw::c_int,
    pub suit: [::std::os::raw::c_int; 52usize],
    pub rank: [::std::os::raw::c_int; 52usize],
}
#[test]
fn bindgen_test_layout_playTraceBin() {
    assert_eq!(
        ::std::mem::size_of::<playTraceBin>(),
        420usize,
        concat!("Size of: ", stringify!(playTraceBin))
    );
    assert_eq!(
        ::std::mem::align_of::<playTraceBin>(),
        4usize,
        concat!("Alignment of ", stringify!(playTraceBin))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTraceBin>())).number as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(playTraceBin),
            "::",
            stringify!(number)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTraceBin>())).suit as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(playTraceBin),
            "::",
            stringify!(suit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTraceBin>())).rank as *const _ as usize },
        212usize,
        concat!(
            "Offset of field: ",
            stringify!(playTraceBin),
            "::",
            stringify!(rank)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct playTracePBN {
    pub number: ::std::os::raw::c_int,
    pub cards: [::std::os::raw::c_char; 106usize],
}
#[test]
fn bindgen_test_layout_playTracePBN() {
    assert_eq!(
        ::std::mem::size_of::<playTracePBN>(),
        112usize,
        concat!("Size of: ", stringify!(playTracePBN))
    );
    assert_eq!(
        ::std::mem::align_of::<playTracePBN>(),
        4usize,
        concat!("Alignment of ", stringify!(playTracePBN))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTracePBN>())).number as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(playTracePBN),
            "::",
            stringify!(number)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTracePBN>())).cards as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(playTracePBN),
            "::",
            stringify!(cards)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct solvedPlay {
    pub number: ::std::os::raw::c_int,
    pub tricks: [::std::os::raw::c_int; 53usize],
}
#[test]
fn bindgen_test_layout_solvedPlay() {
    assert_eq!(
        ::std::mem::size_of::<solvedPlay>(),
        216usize,
        concat!("Size of: ", stringify!(solvedPlay))
    );
    assert_eq!(
        ::std::mem::align_of::<solvedPlay>(),
        4usize,
        concat!("Alignment of ", stringify!(solvedPlay))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<solvedPlay>())).number as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedPlay),
            "::",
            stringify!(number)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<solvedPlay>())).tricks as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedPlay),
            "::",
            stringify!(tricks)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct playTracesBin {
    pub noOfBoards: ::std::os::raw::c_int,
    pub plays: [playTraceBin; 200usize],
}
#[test]
fn bindgen_test_layout_playTracesBin() {
    assert_eq!(
        ::std::mem::size_of::<playTracesBin>(),
        84004usize,
        concat!("Size of: ", stringify!(playTracesBin))
    );
    assert_eq!(
        ::std::mem::align_of::<playTracesBin>(),
        4usize,
        concat!("Alignment of ", stringify!(playTracesBin))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTracesBin>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(playTracesBin),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTracesBin>())).plays as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(playTracesBin),
            "::",
            stringify!(plays)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct playTracesPBN {
    pub noOfBoards: ::std::os::raw::c_int,
    pub plays: [playTracePBN; 200usize],
}
#[test]
fn bindgen_test_layout_playTracesPBN() {
    assert_eq!(
        ::std::mem::size_of::<playTracesPBN>(),
        22404usize,
        concat!("Size of: ", stringify!(playTracesPBN))
    );
    assert_eq!(
        ::std::mem::align_of::<playTracesPBN>(),
        4usize,
        concat!("Alignment of ", stringify!(playTracesPBN))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTracesPBN>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(playTracesPBN),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTracesPBN>())).plays as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(playTracesPBN),
            "::",
            stringify!(plays)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct solvedPlays {
    pub noOfBoards: ::std::os::raw::c_int,
    pub solved: [solvedPlay; 200usize],
}
#[test]
fn bindgen_test_layout_solvedPlays() {
    assert_eq!(
        ::std::mem::size_of::<solvedPlays>(),
        43204usize,
        concat!("Size of: ", stringify!(solvedPlays))
    );
    assert_eq!(
        ::std::mem::align_of::<solvedPlays>(),
        4usize,
        concat!("Alignment of ", stringify!(solvedPlays))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<solvedPlays>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedPlays),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<solvedPlays>())).solved as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedPlays),
            "::",
            stringify!(solved)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DDSInfo {
    pub major: ::std::os::raw::c_int,
    pub minor: ::std::os::raw::c_int,
    pub patch: ::std::os::raw::c_int,
    pub versionString: [::std::os::raw::c_char; 10usize],
    pub system: ::std::os::raw::c_int,
    pub numBits: ::std::os::raw::c_int,
    pub compiler: ::std::os::raw::c_int,
    pub constructor: ::std::os::raw::c_int,
    pub numCores: ::std::os::raw::c_int,
    pub threading: ::std::os::raw::c_int,
    pub noOfThreads: ::std::os::raw::c_int,
    pub threadSizes: [::std::os::raw::c_char; 128usize],
    pub systemString: [::std::os::raw::c_char; 1024usize],
}
#[test]
fn bindgen_test_layout_DDSInfo() {
    assert_eq!(
        ::std::mem::size_of::<DDSInfo>(),
        1204usize,
        concat!("Size of: ", stringify!(DDSInfo))
    );
    assert_eq!(
        ::std::mem::align_of::<DDSInfo>(),
        4usize,
        concat!("Alignment of ", stringify!(DDSInfo))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).major as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(major)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).minor as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(minor)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).patch as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(patch)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).versionString as *const _ as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(versionString)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).system as *const _ as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(system)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).numBits as *const _ as usize },
        28usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(numBits)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).compiler as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(compiler)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).constructor as *const _ as usize },
        36usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(constructor)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).numCores as *const _ as usize },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(numCores)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).threading as *const _ as usize },
        44usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(threading)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).noOfThreads as *const _ as usize },
        48usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(noOfThreads)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).threadSizes as *const _ as usize },
        52usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(threadSizes)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).systemString as *const _ as usize },
        180usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(systemString)
        )
    );
}
extern "C" {
    pub fn SetMaxThreads(userThreads: ::std::os::raw::c_int);
}
extern "C" {
    pub fn SetThreading(code: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SetResources(maxMemoryMB: ::std::os::raw::c_int, maxThreads: ::std::os::raw::c_int);
}
extern "C" {
    pub fn FreeMemory();
}
extern "C" {
    pub fn SolveBoard(
        dl: deal,
        target: ::std::os::raw::c_int,
        solutions: ::std::os::raw::c_int,
        mode: ::std::os::raw::c_int,
        futp: *mut futureTricks,
        threadIndex: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveBoardPBN(
        dlpbn: dealPBN,
        target: ::std::os::raw::c_int,
        solutions: ::std::os::raw::c_int,
        mode: ::std::os::raw::c_int,
        futp: *mut futureTricks,
        thrId: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcDDtable(
        tableDeal: ddTableDeal,
        tablep: *mut ddTableResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcDDtablePBN(
        tableDealPBN: ddTableDealPBN,
        tablep: *mut ddTableResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcAllTables(
        dealsp: *mut ddTableDeals,
        mode: ::std::os::raw::c_int,
        trumpFilter: *mut ::std::os::raw::c_int,
        resp: *mut ddTablesRes,
        presp: *mut allParResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcAllTablesPBN(
        dealsp: *mut ddTableDealsPBN,
        mode: ::std::os::raw::c_int,
        trumpFilter: *mut ::std::os::raw::c_int,
        resp: *mut ddTablesRes,
        presp: *mut allParResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllBoards(bop: *mut boardsPBN, solvedp: *mut solvedBoards)
        -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllBoardsBin(bop: *mut boards, solvedp: *mut solvedBoards)
        -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllChunks(
        bop: *mut boardsPBN,
        solvedp: *mut solvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllChunksBin(
        bop: *mut boards,
        solvedp: *mut solvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllChunksPBN(
        bop: *mut boardsPBN,
        solvedp: *mut solvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Par(
        tablep: *mut ddTableResults,
        presp: *mut parResults,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcPar(
        tableDeal: ddTableDeal,
        vulnerable: ::std::os::raw::c_int,
        tablep: *mut ddTableResults,
        presp: *mut parResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcParPBN(
        tableDealPBN: ddTableDealPBN,
        tablep: *mut ddTableResults,
        vulnerable: ::std::os::raw::c_int,
        presp: *mut parResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SidesPar(
        tablep: *mut ddTableResults,
        sidesRes: *mut parResultsDealer,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn DealerPar(
        tablep: *mut ddTableResults,
        presp: *mut parResultsDealer,
        dealer: ::std::os::raw::c_int,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn DealerParBin(
        tablep: *mut ddTableResults,
        presp: *mut parResultsMaster,
        dealer: ::std::os::raw::c_int,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SidesParBin(
        tablep: *mut ddTableResults,
        sidesRes: *mut parResultsMaster,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn ConvertToDealerTextFormat(
        pres: *mut parResultsMaster,
        resp: *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn ConvertToSidesTextFormat(
        pres: *mut parResultsMaster,
        resp: *mut parTextResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn AnalysePlayBin(
        dl: deal,
        play: playTraceBin,
        solved: *mut solvedPlay,
        thrId: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn AnalysePlayPBN(
        dlPBN: dealPBN,
        playPBN: playTracePBN,
        solvedp: *mut solvedPlay,
        thrId: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn AnalyseAllPlaysBin(
        bop: *mut boards,
        plp: *mut playTracesBin,
        solvedp: *mut solvedPlays,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn AnalyseAllPlaysPBN(
        bopPBN: *mut boardsPBN,
        plpPBN: *mut playTracesPBN,
        solvedp: *mut solvedPlays,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn GetDDSInfo(info: *mut DDSInfo);
}
extern "C" {
    pub fn ErrorMessage(code: ::std::os::raw::c_int, line: *mut ::std::os::raw::c_char);
}
extern "C" {
    pub static mut lho: [::std::os::raw::c_int; 4usize];
}
extern "C" {
    pub static mut rho: [::std::os::raw::c_int; 4usize];
}
extern "C" {
    pub static mut partner: [::std::os::raw::c_int; 4usize];
}
extern "C" {
    pub static mut bitMapRank: [::std::os::raw::c_ushort; 16usize];
}
extern "C" {
    pub static mut cardRank: [::std::os::raw::c_uchar; 16usize];
}
extern "C" {
    pub static mut cardSuit: [::std::os::raw::c_uchar; 5usize];
}
extern "C" {
    pub static mut cardHand: [::std::os::raw::c_uchar; 4usize];
}
extern "C" {
    pub static mut highestRank: [::std::os::raw::c_int; 8192usize];
}
extern "C" {
    pub static mut lowestRank: [::std::os::raw::c_int; 8192usize];
}
extern "C" {
    pub static mut counttable: [::std::os::raw::c_int; 8192usize];
}
extern "C" {
    pub static mut relRank: [[::std::os::raw::c_char; 15usize]; 8192usize];
}
extern "C" {
    pub static mut winRanks: [[::std::os::raw::c_ushort; 14usize]; 8192usize];
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct moveGroupType {
    pub lastGroup: ::std::os::raw::c_int,
    pub rank: [::std::os::raw::c_int; 7usize],
    pub sequence: [::std::os::raw::c_int; 7usize],
    pub fullseq: [::std::os::raw::c_int; 7usize],
    pub gap: [::std::os::raw::c_int; 7usize],
}
#[test]
fn bindgen_test_layout_moveGroupType() {
    assert_eq!(
        ::std::mem::size_of::<moveGroupType>(),
        116usize,
        concat!("Size of: ", stringify!(moveGroupType))
    );
    assert_eq!(
        ::std::mem::align_of::<moveGroupType>(),
        4usize,
        concat!("Alignment of ", stringify!(moveGroupType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveGroupType>())).lastGroup as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(moveGroupType),
            "::",
            stringify!(lastGroup)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveGroupType>())).rank as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(moveGroupType),
            "::",
            stringify!(rank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveGroupType>())).sequence as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(moveGroupType),
            "::",
            stringify!(sequence)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveGroupType>())).fullseq as *const _ as usize },
        60usize,
        concat!(
            "Offset of field: ",
            stringify!(moveGroupType),
            "::",
            stringify!(fullseq)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveGroupType>())).gap as *const _ as usize },
        88usize,
        concat!(
            "Offset of field: ",
            stringify!(moveGroupType),
            "::",
            stringify!(gap)
        )
    );
}
extern "C" {
    pub static mut groupData: [moveGroupType; 8192usize];
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct moveType {
    pub suit: ::std::os::raw::c_int,
    pub rank: ::std::os::raw::c_int,
    pub sequence: ::std::os::raw::c_int,
    pub weight: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_moveType() {
    assert_eq!(
        ::std::mem::size_of::<moveType>(),
        16usize,
        concat!("Size of: ", stringify!(moveType))
    );
    assert_eq!(
        ::std::mem::align_of::<moveType>(),
        4usize,
        concat!("Alignment of ", stringify!(moveType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveType>())).suit as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(moveType),
            "::",
            stringify!(suit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveType>())).rank as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(moveType),
            "::",
            stringify!(rank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveType>())).sequence as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(moveType),
            "::",
            stringify!(sequence)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveType>())).weight as *const _ as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(moveType),
            "::",
            stringify!(weight)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct movePlyType {
    pub move_: [moveType; 14usize],
    pub current: ::std::os::raw::c_int,
    pub last: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_movePlyType() {
    assert_eq!(
        ::std::mem::size_of::<movePlyType>(),
        232usize,
        concat!("Size of: ", stringify!(movePlyType))
    );
    assert_eq!(
        ::std::mem::align_of::<movePlyType>(),
        4usize,
        concat!("Alignment of ", stringify!(movePlyType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<movePlyType>())).move_ as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(movePlyType),
            "::",
            stringify!(move_)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<movePlyType>())).current as *const _ as usize },
        224usize,
        concat!(
            "Offset of field: ",
            stringify!(movePlyType),
            "::",
            stringify!(current)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<movePlyType>())).last as *const _ as usize },
        228usize,
        concat!(
            "Offset of field: ",
            stringify!(movePlyType),
            "::",
            stringify!(last)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct highCardType {
    pub rank: ::std::os::raw::c_int,
    pub hand: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_highCardType() {
    assert_eq!(
        ::std::mem::size_of::<highCardType>(),
        8usize,
        concat!("Size of: ", stringify!(highCardType))
    );
    assert_eq!(
        ::std::mem::align_of::<highCardType>(),
        4usize,
        concat!("Alignment of ", stringify!(highCardType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<highCardType>())).rank as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(highCardType),
            "::",
            stringify!(rank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<highCardType>())).hand as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(highCardType),
            "::",
            stringify!(hand)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pos {
    pub rankInSuit: [[::std::os::raw::c_ushort; 4usize]; 4usize],
    pub aggr: [::std::os::raw::c_ushort; 4usize],
    pub length: [[::std::os::raw::c_uchar; 4usize]; 4usize],
    pub handDist: [::std::os::raw::c_int; 4usize],
    pub winRanks: [[::std::os::raw::c_ushort; 4usize]; 50usize],
    pub first: [::std::os::raw::c_int; 50usize],
    pub move_: [moveType; 50usize],
    pub handRelFirst: ::std::os::raw::c_int,
    pub tricksMAX: ::std::os::raw::c_int,
    pub winner: [highCardType; 4usize],
    pub secondBest: [highCardType; 4usize],
}
#[test]
fn bindgen_test_layout_pos() {
    assert_eq!(
        ::std::mem::size_of::<pos>(),
        1544usize,
        concat!("Size of: ", stringify!(pos))
    );
    assert_eq!(
        ::std::mem::align_of::<pos>(),
        4usize,
        concat!("Alignment of ", stringify!(pos))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).rankInSuit as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(rankInSuit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).aggr as *const _ as usize },
        32usize,
        concat!("Offset of field: ", stringify!(pos), "::", stringify!(aggr))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).length as *const _ as usize },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(length)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).handDist as *const _ as usize },
        56usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(handDist)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).winRanks as *const _ as usize },
        72usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(winRanks)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).first as *const _ as usize },
        472usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(first)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).move_ as *const _ as usize },
        672usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(move_)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).handRelFirst as *const _ as usize },
        1472usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(handRelFirst)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).tricksMAX as *const _ as usize },
        1476usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(tricksMAX)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).winner as *const _ as usize },
        1480usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(winner)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).secondBest as *const _ as usize },
        1512usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(secondBest)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct evalType {
    pub tricks: ::std::os::raw::c_int,
    pub winRanks: [::std::os::raw::c_ushort; 4usize],
}
#[test]
fn bindgen_test_layout_evalType() {
    assert_eq!(
        ::std::mem::size_of::<evalType>(),
        12usize,
        concat!("Size of: ", stringify!(evalType))
    );
    assert_eq!(
        ::std::mem::align_of::<evalType>(),
        4usize,
        concat!("Alignment of ", stringify!(evalType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<evalType>())).tricks as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(evalType),
            "::",
            stringify!(tricks)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<evalType>())).winRanks as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(evalType),
            "::",
            stringify!(winRanks)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct card {
    pub suit: ::std::os::raw::c_int,
    pub rank: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_card() {
    assert_eq!(
        ::std::mem::size_of::<card>(),
        8usize,
        concat!("Size of: ", stringify!(card))
    );
    assert_eq!(
        ::std::mem::align_of::<card>(),
        4usize,
        concat!("Alignment of ", stringify!(card))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<card>())).suit as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(card),
            "::",
            stringify!(suit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<card>())).rank as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(card),
            "::",
            stringify!(rank)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct extCard {
    pub suit: ::std::os::raw::c_int,
    pub rank: ::std::os::raw::c_int,
    pub sequence: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_extCard() {
    assert_eq!(
        ::std::mem::size_of::<extCard>(),
        12usize,
        concat!("Size of: ", stringify!(extCard))
    );
    assert_eq!(
        ::std::mem::align_of::<extCard>(),
        4usize,
        concat!("Alignment of ", stringify!(extCard))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<extCard>())).suit as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(extCard),
            "::",
            stringify!(suit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<extCard>())).rank as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(extCard),
            "::",
            stringify!(rank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<extCard>())).sequence as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(extCard),
            "::",
            stringify!(sequence)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct absRankType {
    pub rank: ::std::os::raw::c_char,
    pub hand: ::std::os::raw::c_schar,
}
#[test]
fn bindgen_test_layout_absRankType() {
    assert_eq!(
        ::std::mem::size_of::<absRankType>(),
        2usize,
        concat!("Size of: ", stringify!(absRankType))
    );
    assert_eq!(
        ::std::mem::align_of::<absRankType>(),
        1usize,
        concat!("Alignment of ", stringify!(absRankType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<absRankType>())).rank as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(absRankType),
            "::",
            stringify!(rank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<absRankType>())).hand as *const _ as usize },
        1usize,
        concat!(
            "Offset of field: ",
            stringify!(absRankType),
            "::",
            stringify!(hand)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct relRanksType {
    pub absRank: [[absRankType; 4usize]; 15usize],
}
#[test]
fn bindgen_test_layout_relRanksType() {
    assert_eq!(
        ::std::mem::size_of::<relRanksType>(),
        120usize,
        concat!("Size of: ", stringify!(relRanksType))
    );
    assert_eq!(
        ::std::mem::align_of::<relRanksType>(),
        1usize,
        concat!("Alignment of ", stringify!(relRanksType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<relRanksType>())).absRank as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(relRanksType),
            "::",
            stringify!(absRank)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct paramType {
    pub noOfBoards: ::std::os::raw::c_int,
    pub bop: *mut boards,
    pub solvedp: *mut solvedBoards,
    pub error: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_paramType() {
    assert_eq!(
        ::std::mem::size_of::<paramType>(),
        32usize,
        concat!("Size of: ", stringify!(paramType))
    );
    assert_eq!(
        ::std::mem::align_of::<paramType>(),
        8usize,
        concat!("Alignment of ", stringify!(paramType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<paramType>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(paramType),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<paramType>())).bop as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(paramType),
            "::",
            stringify!(bop)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<paramType>())).solvedp as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(paramType),
            "::",
            stringify!(solvedp)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<paramType>())).error as *const _ as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(paramType),
            "::",
            stringify!(error)
        )
    );
}
pub const RunMode_DDS_RUN_SOLVE: RunMode = 0;
pub const RunMode_DDS_RUN_CALC: RunMode = 1;
pub const RunMode_DDS_RUN_TRACE: RunMode = 2;
pub const RunMode_DDS_RUN_SIZE: RunMode = 3;
pub type RunMode = ::std::os::raw::c_uint;
