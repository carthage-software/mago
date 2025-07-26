<?php

abstract class UV
{
    public const RUN_DEFAULT = 0;
    public const RUN_ONCE = 1;
    public const RUN_NOWAIT = 2;
    public const CHANGE = 2;
    public const RENAME = 1;
    public const READABLE = 1;
    public const WRITABLE = 2;
    public const O_RDONLY = 0;
    public const O_WRONLY = 1;
    public const O_RDWR = 2;
    public const O_CREAT = 64;
    public const O_EXCL = 128;
    public const O_TRUNC = 512;
    public const O_APPEND = 1024;
    public const S_IFDIR = 16384;
    public const S_IFREG = 32768;
    public const O_NOCTTY = 256;
    public const S_IRWXU = 448;
    public const S_IRUSR = 256;
    public const S_IWUSR = 128;
    public const S_IXUSR = 64;
    public const S_IRWXG = 56;
    public const S_IRGRP = 32;
    public const S_IWGRP = 16;
    public const S_IXGRP = 8;
    public const S_IRWXO = 7;
    public const S_IROTH = 4;
    public const S_IWOTH = 2;
    public const S_IXOTH = 1;
    public const SIG_IGN = 1;
    public const SIG_DFL = 0;
    public const SIG_ERR = -1;
    public const SIGHUP = 1;
    public const SIGINT = 2;
    public const SIGQUIT = 3;
    public const SIGILL = 4;
    public const SIGTRAP = 5;
    public const SIGABRT = 6;
    public const SIGIOT = 6;
    public const SIGBUS = 7;
    public const SIGFPE = 8;
    public const SIGKILL = 9;
    public const SIGUSR1 = 10;
    public const SIGSEGV = 11;
    public const SIGUSR2 = 12;
    public const SIGPIPE = 13;
    public const SIGALRM = 14;
    public const SIGTERM = 15;
    public const SIGSTKFLT = 16;
    public const SIGCHLD = 17;
    public const SIGCONT = 18;
    public const SIGSTOP = 19;
    public const SIGTSTP = 20;
    public const SIGTTIN = 21;
    public const SIGTTOU = 22;
    public const SIGURG = 23;
    public const SIGXCPU = 24;
    public const SIGXFSZ = 25;
    public const SIGVTALRM = 26;
    public const SIGPROF = 27;
    public const SIGWINCH = 28;
    public const SIGPOLL = 29;
    public const SIGIO = 29;
    public const SIGPWR = 30;
    public const SIGSYS = 31;
    public const SIGBABY = 31;
    public const AF_INET = 2;
    public const AF_INET6 = 10;
    public const AF_UNSPEC = 0;
    public const LEAVE_GROUP = 0;
    public const JOIN_GROUP = 1;
    public const IS_UV_TCP = 0;
    public const IS_UV_UDP = 1;
    public const IS_UV_PIPE = 2;
    public const IS_UV_IDLE = 3;
    public const IS_UV_TIMER = 4;
    public const IS_UV_ASYNC = 5;
    public const IS_UV_LOOP = 6;
    public const IS_UV_HANDLE = 7;
    public const IS_UV_STREAM = 8;
    public const IS_UV_ADDRINFO = 9;
    public const IS_UV_PROCESS = 10;
    public const IS_UV_PREPARE = 11;
    public const IS_UV_CHECK = 12;
    public const IS_UV_WORK = 13;
    public const IS_UV_FS = 14;
    public const IS_UV_FS_EVENT = 15;
    public const IS_UV_TTY = 16;
    public const IS_UV_FS_POLL = 17;
    public const IS_UV_POLL = 18;
    public const UNKNOWN_HANDLE = 0;
    public const FILE = 17;
    public const ASYNC = 1;
    public const CHECK = 2;
    public const FS_EVENT = 3;
    public const FS_POLL = 4;
    public const HANDLE = 5;
    public const IDLE = 6;
    public const NAMED_PIPE = 7;
    public const POLL = 8;
    public const PREPARE = 9;
    public const PROCESS = 10;
    public const STREAM = 11;
    public const TCP = 12;
    public const TIMER = 13;
    public const TTY = 14;
    public const UDP = 15;
    public const SIGNAL = 16;
    public const HANDLE_TYPE_MAX = 18;
    public const IGNORE = 0;
    public const CREATE_PIPE = 1;
    public const INHERIT_FD = 2;
    public const INHERIT_STREAM = 4;
    public const READABLE_PIPE = 16;
    public const WRITABLE_PIPE = 32;
    public const PROCESS_SETUID = 1;
    public const PROCESS_SETGID = 2;
    public const PROCESS_WINDOWS_VERBATIM_ARGUMENTS = 4;
    public const PROCESS_DETACHED = 8;
    public const E2BIG = -7;
    public const EACCES = -13;
    public const EADDRINUSE = -98;
    public const EADDRNOTAVAIL = -99;
    public const EAFNOSUPPORT = -97;
    public const EAGAIN = -11;
    public const EAI_ADDRFAMILY = -3000;
    public const EAI_AGAIN = -3001;
    public const EAI_BADFLAGS = -3002;
    public const EAI_BADHINTS = -3013;
    public const EAI_CANCELED = -3003;
    public const EAI_FAIL = -3004;
    public const EAI_FAMILY = -3005;
    public const EAI_MEMORY = -3006;
    public const EAI_NODATA = -3007;
    public const EAI_NONAME = -3008;
    public const EAI_OVERFLOW = -3009;
    public const EAI_PROTOCOL = -3014;
    public const EAI_SERVICE = -3010;
    public const EAI_SOCKTYPE = -3011;
    public const EALREADY = -114;
    public const EBADF = -9;
    public const EBUSY = -16;
    public const ECANCELED = -125;
    public const ECHARSET = -4080;
    public const ECONNABORTED = -103;
    public const ECONNREFUSED = -111;
    public const ECONNRESET = -104;
    public const EDESTADDRREQ = -89;
    public const EEXIST = -17;
    public const EFAULT = -14;
    public const EFBIG = -27;
    public const EHOSTUNREACH = -113;
    public const EINTR = -4;
    public const EINVAL = -22;
    public const EIO = -5;
    public const EISCONN = -106;
    public const EISDIR = -21;
    public const ELOOP = -40;
    public const EMFILE = -24;
    public const EMSGSIZE = -90;
    public const ENAMETOOLONG = -36;
    public const ENETDOWN = -100;
    public const ENETUNREACH = -101;
    public const ENFILE = -23;
    public const ENOBUFS = -105;
    public const ENODEV = -19;
    public const ENOENT = -2;
    public const ENOMEM = -12;
    public const ENONET = -64;
    public const ENOPROTOOPT = -92;
    public const ENOSPC = -28;
    public const ENOSYS = -38;
    public const ENOTCONN = -107;
    public const ENOTDIR = -20;
    public const ENOTEMPTY = -39;
    public const ENOTSOCK = -88;
    public const ENOTSUP = -95;
    public const EPERM = -1;
    public const EPIPE = -32;
    public const EPROTO = -71;
    public const EPROTONOSUPPORT = -93;
    public const EPROTOTYPE = -91;
    public const ERANGE = -34;
    public const EROFS = -30;
    public const ESHUTDOWN = -108;
    public const ESPIPE = -29;
    public const ESRCH = -3;
    public const ETIMEDOUT = -110;
    public const ETXTBSY = -26;
    public const EXDEV = -18;
    public const UNKNOWN = -4094;
    public const EOF = -4095;
    public const ENXIO = -6;
    public const EMLINK = -31;
    public const EHOSTDOWN = -112;
    public const EREMOTEIO = -121;
    public const ENOTTY = -25;
    public const EFTYPE = -4028;
}

abstract class UVStream extends UV
{
}

class UVTcp extends UVStream
{
}

class UVUdp extends UV
{
}

class UVPipe extends UVStream
{
}

class UVIdle extends UV
{
}

class UVTimer extends UV
{
}

class UVAsync extends UV
{
}

class UVAddrinfo extends UV
{
}

class UVProcess extends UV
{
}

class UVPrepare extends UV
{
}

class UVCheck extends UV
{
}

class UVWork extends UV
{
}

class UVFs extends UV
{
}

class UVFsEvent extends UV
{
}

class UVTty extends UVStream
{
}

class UVFsPoll extends UV
{
}

class UVPoll extends UV
{
}

class UVSignal extends UV
{
}

class UVLoop
{
}

abstract class UVSockAddr
{
}

class UVSockAddrIPv4 extends UVSockAddr
{
}

class UVSockAddrIPv6 extends UVSockAddr
{
}

class UVLock
{
}

class UVStdio
{
}

function uv_unref(UV $uv_t): void
{
}

function uv_last_error(null|UVLoop $uv_loop = null): int
{
}

function uv_err_name(int $error_code): string
{
}

function uv_strerror(int $error_code): string
{
}

function uv_update_time(UVLoop $uv_loop): void
{
}

function uv_ref(UV $uv_handle): void
{
}

function uv_run(null|UVLoop $uv_loop = null, int $run_mode = UV::RUN_DEFAULT): void
{
}

function uv_run_once(UVLoop|null $uv_loop = null): void
{
}

function uv_loop_delete(UVLoop $uv_loop): void
{
}

function uv_now(): int
{
}

function uv_tcp_bind(UVTcp $uv_tcp, UVSockAddr $uv_sockaddr): void
{
}

function uv_tcp_bind6(UVTcp $uv_tcp, UVSockAddr $uv_sockaddr): void
{
}

function uv_write(UVStream $handle, string $data, callable $callback): void
{
}

function uv_write2(UVStream $handle, string $data, UVTcp|UvPipe $send, callable $callback): void
{
}

function uv_tcp_nodelay(UVTcp $handle, bool $enable)
{
}

function uv_accept(UVTcp|UVPipe $server, UVTcp|UVPipe $client): void
{
}

function uv_shutdown(UVStream $handle, callable $callback): void
{
}

function uv_close(UV $handle, null|callable $callback = null): void
{
}

function uv_read_start(UVStream $handle, callable $callback)
{
}

function uv_read_stop(UVStream $handle): void
{
}

function uv_ip4_addr(string $ipv4_addr, int $port): UVSockAddrIPv4
{
}

function uv_ip6_addr(string $ipv6_addr, int $port): UVSockAddrIPv6
{
}

function uv_listen(UVTcp|UVPipe $handle, int $backlog, callable $callback): void
{
}

function uv_tcp_connect(UVTcp $handle, UVSockAddr $ipv4_addr, callable $callback): void
{
}

function uv_tcp_connect6(UVTcp $handle, UVSockAddrIPv6 $ipv6_addr, callable $callback): void
{
}

function uv_timer_init(UVLoop $loop = null): UVTimer
{
}

function uv_timer_start(UVTimer $timer, int $timeout, int $repeat, callable $callback): void
{
}

function uv_timer_stop(UVTimer $timer): int
{
}

function uv_timer_again(UVTimer $timer): void
{
}

function uv_timer_set_repeat(UVTimer $timer, int $repeat): void
{
}

function uv_timer_get_repeat(UVTimer $timer): int
{
}

function uv_idle_init(null|UVLoop $loop = null): UVIdle
{
}

function uv_idle_start(UVIdle $idle, callable $callback): void
{
}

/**
 * @param UVIdle $idle
 */
function uv_idle_stop($idle): void
{
}

/**
 * @param UVLoop $loop
 */
function uv_getaddrinfo(UVLoop $loop, callable $callback, string $node, string $service, array $hints): void
{
}

/**
 * @param UVLoop|null $loop
 *
 * @return UVTcp
 */
function uv_tcp_init($loop = null)
{
}

function uv_default_loop(): UVLoop
{
}

function uv_loop_new(): UVLoop
{
}

function uv_udp_init(UVLoop|null $loop = null): UVUdp
{
}

function uv_udp_bind(UVUdp $resource, UVSockAddr $address, int $flags): void
{
}

function uv_udp_bind6(UVUdp $resource, UVSockAddr $address, int $flags): void
{
}

function uv_udp_recv_start(UVUdp $handle, callable $callback): void
{
}

function uv_udp_recv_stop(UVUdp $handle): void
{
}

function uv_udp_set_membership(UVUdp $handle, string $multicast_addr, string $interface_addr, int $membership): int
{
}

function uv_udp_set_multicast_loop(UVUdp $handle, int $enabled): void
{
}

function uv_udp_set_multicast_ttl(UVUdp $handle, int $ttl): void
{
}

function uv_udp_set_broadcast(UVUdp $handle, bool $enabled): void
{
}

function uv_udp_send(UVUdp $handle, string $data, UVSockAddr $uv_addr, callable $callback): void
{
}

function uv_udp_send6(UVUdp $handle, string $data, UVSockAddrIPv6 $uv_addr6, callable $callback): void
{
}

function uv_is_active(UV $handle): bool
{
}

function uv_is_closing(UV $handle): bool
{
}

function uv_is_readable(UVStream $handle): bool
{
}

function uv_is_writable(UVStream $handle): bool
{
}

function uv_walk(UVLoop $loop, callable $closure, array $opaque = null): bool
{
}

/**
 * @param resource $uv
 */
function uv_guess_handle($uv): int
{
}

function uv_pipe_init(UVLoop $loop, int $ipc): UVPipe
{
}

function uv_pipe_open(UVPipe $handle, int $pipe): void
{
}

function uv_pipe_bind(UVPipe $handle, string $name): int
{
}

function uv_pipe_connect(UVPipe $handle, string $path, callable $callback): void
{
}

function uv_pipe_pending_instances(UVPipe $handle, $count): void
{
}

function uv_loadavg(): array
{
}

function uv_uptime(): float
{
}

function uv_get_free_memory(): int
{
}

function uv_get_total_memory(): int
{
}

function uv_hrtime(): int
{
}

function uv_exepath(): string
{
}

function uv_cpu_info(): array
{
}

function uv_interface_addresses(): array
{
}

/**
 * @param UV|resource|int|null $fd
 */
function uv_stdio_new($fd, int $flags): UVStdio
{
}

function uv_spawn(
    UVLoop $loop,
    string $command,
    array $args,
    array $stdio,
    string $cwd,
    array $env = [],
    null|callable $callback = null,
    null|int $flags = null,
    null|array $options = null,
): UVProcess|int {
}

function uv_process_kill(UVProcess $handle, int $signal): void
{
}

function uv_kill(int $pid, int $signal)
{
}

function uv_chdir(string $directory): bool
{
}

function uv_rwlock_init(): UVLock
{
}

function uv_rwlock_rdlock(UVLock $handle)
{
}

function uv_rwlock_tryrdlock(UVLock $handle): bool
{
}

function uv_rwlock_rdunlock(UVLock $handle): void
{
}

function uv_rwlock_wrlock(UVLock $handle): void
{
}

function uv_rwlock_trywrlock(UVLock $handle)
{
}

function uv_rwlock_wrunlock(UVLock $handle)
{
}

function uv_mutex_init(): UVLock
{
}

function uv_mutex_lock(UVLock $lock): void
{
}

function uv_mutex_trylock(UVLock $lock): bool
{
}

function uv_sem_init(int $value): UVLock
{
}

function uv_sem_post(UVLock $sem): void
{
}

function uv_sem_wait(UVLock $sem): void
{
}

function uv_sem_trywait(UVLock $sem): void
{
}

function uv_prepare_init(UVLoop $loop): UVPrepare
{
}

function uv_prepare_start(UVPrepare $handle, callable $callback): void
{
}

function uv_prepare_stop(UVPrepare $handle): void
{
}

function uv_check_init(UVLoop $loop): UVCheck
{
}

function uv_check_start(UVCheck $handle, callable $callback): void
{
}

function uv_check_stop(UVCheck $handle): void
{
}

function uv_async_init(UVLoop $loop, callable $callback): UVAsync
{
}

function uv_async_send(UVAsync $handle): void
{
}

function uv_queue_work(UVLoop $loop, callable $callback, callable $after_callback): void
{
}

/**
 * @return resource
 */
function uv_fs_open(UVLoop $loop, string $path, int $flag, int $mode, callable $callback)
{
}

/**
 * @param resource $fd
 */
function uv_fs_read(UVLoop $loop, $fd, int $offset, int $length, callable $callback): void
{
}

/**
 * @param resource $fd
 */
function uv_fs_close(UVLoop $loop, $fd, callable $callback): void
{
}

/**
 * @param resource $fd
 */
function uv_fs_write(UVLoop $loop, $fd, string $buffer, int $offset, callable $callback): void
{
}

/**
 * @param resource $fd
 */
function uv_fs_fsync(UVLoop $loop, $fd, callable $callback): void
{
}

/**
 * @param resource $fd
 */
function uv_fs_fdatasync(UVLoop $loop, $fd, callable $callback): void
{
}

/**
 * @param resource $fd
 */
function uv_fs_ftruncate(UVLoop $loop, $fd, int $offset, callable $callback): void
{
}

function uv_fs_mkdir(UVLoop $loop, string $path, int $mode, callable $callback): void
{
}

function uv_fs_rmdir(UVLoop $loop, string $path, callable $callback): void
{
}

function uv_fs_unlink(UVLoop $loop, string $path, callable $callback): void
{
}

function uv_fs_rename(UVLoop $loop, string $from, string $to, callable $callback): void
{
}

function uv_fs_utime(UVLoop $loop, string $path, int $utime, int $atime, callable $callback): void
{
}

/**
 * @param resource $fd
 */
function uv_fs_futime(UVLoop $loop, $fd, int $utime, int $atime, callable $callback): void
{
}

function uv_fs_chmod(UVLoop $loop, string $path, int $mode, callable $callback): void
{
}

/**
 * @param resource $fd
 */
function uv_fs_fchmod(UVLoop $loop, $fd, int $mode, callable $callback): void
{
}

function uv_fs_chown(UVLoop $loop, string $path, int $uid, int $gid, callable $callback): void
{
}

/**
 * @param resource $fd
 */
function uv_fs_fchown(UVLoop $loop, $fd, int $uid, int $gid, callable $callback): void
{
}

function uv_fs_link(UVLoop $loop, string $from, string $to, callable $callback): void
{
}

function uv_fs_symlink(UVLoop $loop, string $from, string $to, int $flags, callable $callback): void
{
}

function uv_fs_readlink(UVLoop $loop, string $path, callable $callback): void
{
}

function uv_fs_stat(UVLoop $loop, string $path, callable $callback): void
{
}

function uv_fs_lstat(UVLoop $loop, string $path, callable $callback): void
{
}

/**
 * @param resource $fd
 */
function uv_fs_fstat(UVLoop $loop, $fd, callable $callback): void
{
}

function uv_fs_readdir(UVLoop $loop, string $path, int $flags, callable $callback): void
{
}

/**
 * @param resource $in_fd
 * @param resource $out_fd
 */
function uv_fs_sendfile(UVLoop $loop, $in_fd, $out_fd, int $offset, int $length, callable $callback)
{
}

function uv_fs_event_init(UVLoop $loop, string $path, callable $callback, int $flags = 0): UVFsEvent
{
}

/**
 * @param resource $fd
 */
function uv_tty_init(UVLoop $loop, $fd, int $readable): UVTty
{
}

function uv_tty_get_winsize(UVTty $tty, int &$width, int &$height): int
{
}

function uv_tty_set_mode(UVTty $tty, int $mode): int
{
}

function uv_tty_reset_mode(): void
{
}

function uv_tcp_getsockname(UVTcp $uv_sockaddr): string
{
}

function uv_tcp_getpeername(UVTcp $uv_sockaddr): string
{
}

function uv_udp_getsockname(UVUdp $uv_sockaddr): string
{
}

function uv_resident_set_memory(): int
{
}

function uv_ip4_name(UVSockAddr $address): string
{
}

function uv_ip6_name(UVSockAddr $address): string
{
}

/**
 * @param resource $fd
 */
function uv_poll_init(UVLoop $uv_loop, $fd): UVPoll
{
}

function uv_poll_start(UVPoll $handle, int $events, callable $callback): void
{
}

function uv_poll_stop(UVPoll $poll): void
{
}

function uv_fs_poll_init(null|UVLoop $uv_loop = null): UVFsPoll
{
}

function uv_fs_poll_start(UVFsPoll $handle, callable $callback, string $path, int $interval): UV
{
}

function uv_fs_poll_stop(UVFsPoll $poll): void
{
}

function uv_stop(UVLoop $uv_loop): void
{
}

function uv_signal_stop(UVSignal $sig_handle): int
{
}
