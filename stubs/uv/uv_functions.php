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

/**
 * Binds a name to a socket.
 *
 * @param UVTcp $uv_tcp
 * @param UVSockAddr $uv_sockaddr
 *
 * @return void
 */
function uv_tcp_bind6($uv_tcp, $uv_sockaddr)
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

/**
 * Connect to specified ip address and port.
 *
 * @param UVTcp $handle requires uv_tcp_init().
 * @param UVSockAddr $ipv4_addr requires uv_sockaddr.
 * @param callable $callback callable variables. This callback expects (UVTcp $tcp_handle, $status).
 *
 * @return void
 */
function uv_tcp_connect($handle, $ipv4_addr, callable $callback)
{
}

/**
 * Connect to specified ip address and port.
 *
 * @param UVTcp $handle requires uv_tcp_init().
 * @param UVSockAddrIPv6 $ipv6_addr requires uv_sockaddr.
 * @param callable $callback callable variables. This callback expects (UVTcp $tcp_handle, $status).
 *
 * @return void
 */
function uv_tcp_connect6($handle, $ipv6_addr, callable $callback)
{
}

/**
 * Initialize timer handle.
 *
 * @param UVLoop|null $loop uv_loop handle.
 *
 * @return UVTimer
 */
function uv_timer_init($loop = null)
{
}

/**
 * Initialize timer handle.
 *
 * @param UVTimer $timer timer.
 * @param int $timeout periodical event starts when after this timeout. 1000 is 1 sec.
 * @param int $repeat repeat interval. 1000 is 1 sec.
 * @param callable $callback
 *
 * @return void
 */
function uv_timer_start($timer, int $timeout, int $repeat, callable $callback)
{
}

/**
 * stop specified timer.
 *
 * @param UVTimer $timer uv timer.
 *
 * @return int
 */
function uv_timer_stop($timer): int
{
}

/**
 * Restart timer.
 *
 * @param UVTimer $timer uv_timer.
 *
 * @return void
 */
function uv_timer_again($timer)
{
}

/**
 * Set repeat count.
 *
 * @param UVTimer $timer uv_timer.
 * @param int $repeat repeat count.
 *
 * @return void
 */
function uv_timer_set_repeat($timer, int $repeat)
{
}

/**
 * Returns repeat interval.
 *
 * @param UVTimer $timer uv_timer.
 *
 * @return int
 */
function uv_timer_get_repeat($timer): int
{
}

/**
 * Initialize uv idle handle.
 *
 * @param UVLoop $loop uv_loop handle.
 *
 * @return UVIdle initialized idle handle.
 */
function uv_idle_init($loop = null)
{
}

/**
 * start idle callback.
 *
 * @param UVIdle $idle uv_idle.
 * @param callable $callback idle callback.
 *
 * @return void
 */
function uv_idle_start($idle, callable $callback)
{
}

/**
 * Stop idle callback.
 *
 * @param UVIdle $idle uv_idle.
 *
 * @return void
 */
function uv_idle_stop($idle)
{
}

/**
 * @param UVLoop $loop
 * @param callable $callback
 * @param string $node
 * @param string $service
 * @param array $hints
 *
 * @return void
 */
function uv_getaddrinfo($loop, callable $callback, string $node, string $service, array $hints)
{
}

/**
 * Create a tcp socket.
 *
 * @param UVLoop|null $loop loop or null. if not specified loop then use uv_default_loop.
 *
 * @return UVTcp uv handle which initialized for tcp.
 */
function uv_tcp_init($loop = null)
{
}

/**
 * Return default loop handle.
 *
 * @return UVLoop
 */
function uv_default_loop()
{
}

/**
 * Create a new loop handle.
 *
 * @return UVLoop
 */
function uv_loop_new()
{
}

/**
 * Create a udp socket.
 *
 * @param UVLoop|null $loop loop or null. if not specified loop then use uv_default_loop.
 *
 * @return UVUdp uv handle which initialized for udp.
 */
function uv_udp_init($loop = null)
{
}

/**
 * Listens for a connection on a uv udp handle.
 *
 * @param UVUdp $resource uv udp handle.
 * @param UVSockAddr $address uv sockaddr(ipv4).
 * @param int $flags unused.
 *
 * @return void
 */
function uv_udp_bind($resource, $address, int $flags)
{
}

/**
 * Listens for a connection on a uv udp handle.
 *
 * @param UVUdp $resource uv udp handle.
 * @param UVSockAddr $address uv sockaddr(ipv6).
 * @param int $flags Should be 0 or UV::UDP_IPV6ONLY.
 *
 * @return void
 */
function uv_udp_bind6($resource, $address, int $flags)
{
}

/**
 * Start receive callback.
 *
 * @param UVUdp $handle uv udp handle.
 * @param callable $callback this callback parameter expects (UVUdp $stream, long $nread, string $buffer)..
 *
 * @return void
 */
function uv_udp_recv_start($handle, callable $callback)
{
}

/**
 * Stop receive callback.
 *
 * @param UVUdp $handle
 *
 * @return void
 */
function uv_udp_recv_stop($handle)
{
}

/**
 * Join or leave udp muticast group.
 *
 * @param UVUdp $handle uv udp handle.
 * @param string $multicast_addr multicast address.
 * @param string $interface_addr interface address.
 * @param int $membership UV::JOIN_GROUP or UV::LEAVE_GROUP
 *
 * @return int
 */
function uv_udp_set_membership($handle, string $multicast_addr, string $interface_addr, int $membership): int
{
}

/**
 * Set multicast loop.
 *
 * @param UVUdp $handle uv udp handle.
 * @param int $enabled
 *
 * @return void
 */
function uv_udp_set_multicast_loop($handle, int $enabled)
{
}

/**
 * Set multicast ttl.
 *
 * @param UVUdp $handle uv udp handle.
 * @param int $ttl multicast ttl.
 *
 * @return void
 */
function uv_udp_set_multicast_ttl($handle, int $ttl)
{
}

/**
 * Set udp broadcast.
 *
 * @param UVUdp $handle uv udp handle.
 * @param bool $enabled
 *
 * @return void
 */
function uv_udp_set_broadcast($handle, bool $enabled)
{
}

/**
 * Send buffer to specified address.
 *
 * @param UVUdp $handle uv udp handle.
 * @param string $data data.
 * @param UVSockAddr $uv_addr uv_ip4_addr.
 * @param callable $callback this callback parameter expects (UVUdp $stream, long $status).
 *
 * @return void
 */
function uv_udp_send($handle, string $data, $uv_addr, callable $callback)
{
}

/**
 * Send buffer to specified address.
 *
 * @param UVUdp $handle uv udp handle.
 * @param string $data data.
 * @param UVSockAddrIPv6 $uv_addr6 uv_ip6_addr.
 * @param callable $callback this callback parameter expects (UVUdp $stream, long $status).
 *
 * @return void
 */
function uv_udp_send6($handle, string $data, $uv_addr6, callable $callback)
{
}

/**
 * @param UV $handle
 *
 * @return bool
 */
function uv_is_active($handle): bool
{
}

/**
 * @param UV $handle
 *
 * @return bool
 */
function uv_is_closing($handle): bool
{
}

/**
 * @param UVStream $handle
 *
 * @return bool
 */
function uv_is_readable($handle): bool
{
}

/**
 * @param UVStream $handle
 *
 * @return bool
 */
function uv_is_writable($handle): bool
{
}

/**
 * @param UVLoop $loop
 * @param callable $closure
 * @param array|null $opaque
 *
 * @return bool
 */
function uv_walk($loop, callable $closure, array $opaque = null): bool
{
}

/**
 * @param resource $uv
 *
 * @return int
 */
function uv_guess_handle($uv): int
{
}

/**
 * Initialize pipe.
 *
 * @param UVLoop $loop uv_loop handle.
 * @param int $ipc when this pipe use for ipc, please set true otherwise false.
 *
 * @return UVPipe
 */
function uv_pipe_init($loop, int $ipc)
{
}

/**
 * Open a pipe.
 *
 * @param UVPipe $handle uv pipe handle.
 * @param int $pipe dunnno. maybe file descriptor.
 *
 * @return void
 */
function uv_pipe_open($handle, int $pipe)
{
}

/**
 * Create a named pipe.
 *
 * @param UVPipe $handle uv pipe handle.
 * @param string $name dunnno. maybe file descriptor.
 *
 * @return int
 */
function uv_pipe_bind($handle, string $name): int
{
}

/**
 * Connect to named pipe.
 *
 * @param UVPipe $handle uv pipe handle.
 * @param string $path named pipe path.
 * @param callable $callback this callback parameter expects (UVPipe $pipe, long $status).
 *
 * @return void
 */
function uv_pipe_connect($handle, string $path, callable $callback)
{
}

/**
 * @param UVPipe $handle
 * @param void $count
 *
 * @return void
 */
function uv_pipe_pending_instances($handle, $count)
{
}

/**
 * Returns current loadaverage.
 *
 * Note: returns array on windows box. (does not support load average on windows).
 *
 * @return array
 */
function uv_loadavg(): array
{
}

/**
 * Returns current uptime.
 *
 * @return float
 */
function uv_uptime(): float
{
}

/**
 * Returns current free memory size.
 *
 * @return int
 */
function uv_get_free_memory(): int
{
}

/**
 * Returns total memory size.
 *
 * @return int
 */
function uv_get_total_memory(): int
{
}

/**
 * @return int
 */
function uv_hrtime(): int
{
}

/**
 * Returns current exepath. basically this will returns current php path.
 *
 * @return string
 */
function uv_exepath(): string
{
}

/**
 * Returns current cpu informations.
 *
 * @return array
 */
function uv_cpu_info(): array
{
}

/**
 * @return array
 */
function uv_interface_addresses(): array
{
}

/**
 * @param UV|resource|long|null $fd
 * @param int $flags
 *
 * @return UVStdio
 */
function uv_stdio_new($fd, int $flags)
{
}

/**
 * @param UVLoop $loop
 * @param string $command
 * @param array $args
 * @param array $stdio
 * @param string $cwd
 * @param array $env
 * @param callable|null $callback
 * @param int|null $flags
 * @param array|null $options
 *
 * @return UVProcess|long
 */
function uv_spawn(
    $loop,
    string $command,
    array $args,
    array $stdio,
    string $cwd,
    array $env = [],
    null|callable $callback = null,
    null|int $flags = null,
    null|array $options = null,
) {
}

/**
 * Send signal to specified uv process.
 *
 * @param UVProcess $handle uv process handle.
 * @param int $signal
 *
 * @return void
 */
function uv_process_kill($handle, int $signal)
{
}

/**
 * Send signal to specified pid.
 *
 * @param int $pid process id.
 * @param int $signal
 */
function uv_kill(int $pid, int $signal)
{
}

/**
 * Change working directory.
 *
 * @param string $directory
 * @return bool
 */
function uv_chdir(string $directory): bool
{
}

/**
 * Initialize rwlock.
 *
 * @return UVLock returns uv rwlock.
 */
function uv_rwlock_init()
{
}

/**
 * Set read lock.
 *
 * @param UVLock $handle uv lock handle.
 */
function uv_rwlock_rdlock($handle)
{
}

/**
 * @param UVLock $handle
 *
 * @return bool
 */
function uv_rwlock_tryrdlock($handle): bool
{
}

/**
 * Unlock read lock.
 *
 * @param UVLock $handle uv lock handle.
 *
 * @return void
 */
function uv_rwlock_rdunlock($handle)
{
}

/**
 * Set write lock.
 *
 * @param UVLock $handle uv lock handle.
 *
 * @return void
 */
function uv_rwlock_wrlock($handle)
{
}

/**
 * @param UVLock $handle uv lock handle.
 */
function uv_rwlock_trywrlock($handle)
{
}

/**
 * Unlock write lock.
 *
 * @param UVLock $handle uv lock handle.
 */
function uv_rwlock_wrunlock($handle)
{
}

/**
 * Initialize mutex.
 *
 * @return UVLock uv mutex
 */
function uv_mutex_init()
{
}

/**
 * Lock mutex.
 *
 * @param UVLock $lock uv lock handle.
 *
 * @return void
 */
function uv_mutex_lock($lock)
{
}

/**
 * @param UVLock $lock
 *
 * @return bool
 */
function uv_mutex_trylock($lock): bool
{
}

/**
 * Initialize semaphore.
 *
 * @param int $value
 * @return UVLock
 */
function uv_sem_init(int $value)
{
}

/**
 * Post semaphore.
 *
 * @param UVLock $sem uv lock handle.
 *
 * @return void
 */
function uv_sem_post($sem)
{
}

/**
 * @param UVLock $sem
 *
 * @return void
 */
function uv_sem_wait($sem)
{
}

/**
 * @param UVLock $sem
 *
 * @return void
 */
function uv_sem_trywait($sem)
{
}

/**
 * Initialize prepare.
 *
 * @param UVLoop $loop uv loop handle.
 *
 * @return UVPrepare
 */
function uv_prepare_init($loop)
{
}

/**
 * Setup prepare loop callback. (pre loop callback)
 *
 * @param UVPrepare $handle uv prepare handle.
 * @param callable $callback this callback parameter expects (UVPrepare $prepare, long $status).
 *
 * @return void
 */
function uv_prepare_start($handle, callable $callback)
{
}

/**
 * Stop prepare callback.
 *
 * @param UVPrepare $handle uv prepare handle.
 *
 * @return void
 */
function uv_prepare_stop($handle)
{
}

/**
 * Setup check.
 *
 * @param UVLoop $loop uv loop handle
 *
 * @return UVCheck
 */
function uv_check_init($loop)
{
}

/**
 * Stats check loop callback. (after loop callback)
 *
 * @param UVCheck $handle uv check handle.
 * @param callable $callback this callback parameter expects (UVCheck $check, long $status).
 *
 * @return void
 */
function uv_check_start($handle, callable $callback)
{
}

/**
 * Stop check callback.
 *
 * @param UVCheck $handle uv check handle.
 *
 * @return void
 */
function uv_check_stop($handle)
{
}

/**
 * Setup async callback.
 *
 * @param UVLoop $loop uv loop
 * @param callable $callback
 *
 * @return UVAsync uv async handle.
 */
function uv_async_init($loop, callable $callback)
{
}

/**
 * Send async callback immediately.
 *
 * @param UVAsync $handle uv async handle.
 *
 * @return void
 */
function uv_async_send($handle)
{
}

/**
 * Execute callbacks in another thread (requires Thread Safe enabled PHP).
 *
 * @param UVLoop $loop
 * @param callable $callback
 * @param callable $after_callback
 *
 * @return void
 */
function uv_queue_work($loop, callable $callback, callable $after_callback)
{
}

/**
 * Open specified file.
 *
 * @param UVLoop $loop uv_loop handle.
 * @param string $path file path
 * @param int $flag file flag. this should be UV::O_RDONLY and some constants flag.
 * @param int $mode mode flag. this should be UV::S_IRWXU and some mode flag.
 * @param callable $callback this callback parameter expects (resource $stream).
 *
 * @return resource
 */
function uv_fs_open($loop, string $path, int $flag, int $mode, callable $callback)
{
}

/**
 * Async read.
 *
 * @param UVLoop $loop uv loop handle
 * @param resource $fd this expects long $fd, resource $php_stream or resource $php_socket.
 * @param int $offset the offset position in the file at which reading should commence.
 * @param int $length the length in bytes that should be read starting at position $offset.
 * @param callable $callback this callback parameter expects (zval $fd, long $nread, string $buffer).
 *
 * @return void
 */
function uv_fs_read($loop, $fd, int $offset, int $length, callable $callback)
{
}

/**
 * Close specified file descriptor.
 *
 * @param UVLoop $loop uv_loop handle.
 * @param resource $fd file descriptor. this expects long $fd, resource $php_stream or resource $php_socket.
 * @param callable $callback this callback parameter expects (resource $stream)
 *
 * @return void
 */
function uv_fs_close($loop, $fd, callable $callback)
{
}

/**
 * Write buffer to specified file descriptor.
 *
 * @param UVLoop $loop uv_loop handle.
 * @param resource $fd file descriptor. this expects long $fd, resource $php_stream or resource $php_socket.
 * @param string $buffer buffer.
 * @param int $offset
 * @param callable $callback this callback parameter expects (resource $stream, long $result)
 *
 * @return void
 */
function uv_fs_write($loop, $fd, string $buffer, int $offset, callable $callback)
{
}

/**
 * Async fsync.
 *
 * @param UVLoop $loop uv loop handle.
 * @param resource $fd
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_fsync($loop, $fd, callable $callback)
{
}

/**
 * Async fdatasync.
 *
 * @param UVLoop $loop uv loop handle.
 * @param resource $fd
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_fdatasync($loop, $fd, callable $callback)
{
}

/**
 * Async ftruncate.
 *
 * @param UVLoop $loop uv loop handle.
 * @param resource $fd
 * @param int $offset
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_ftruncate($loop, $fd, int $offset, callable $callback)
{
}

/**
 * Async mkdir.
 *
 * @param UVLoop $loop uv loop handle
 * @param string $path
 * @param int $mode
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_mkdir($loop, string $path, int $mode, callable $callback)
{
}

/**
 * Async rmdir.
 *
 * @param UVLoop $loop uv loop handle
 * @param string $path
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_rmdir($loop, string $path, callable $callback)
{
}

/**
 * Async unlink.
 *
 * @param UVLoop $loop uv loop handle
 * @param string $path
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_unlink($loop, string $path, callable $callback)
{
}

/**
 * Async rename.
 *
 * @param UVLoop $loop uv loop handle.
 * @param string $from
 * @param string $to
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_rename($loop, string $from, string $to, callable $callback)
{
}

/**
 * Async utime.
 *
 * @param UVLoop $loop uv loop handle.
 * @param string $path
 * @param int $utime
 * @param int $atime
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_utime($loop, string $path, int $utime, int $atime, callable $callback)
{
}

/**
 * Async futime.
 *
 * @param UVLoop $loop uv loop handle.
 * @param resource $fd
 * @param int $utime
 * @param int $atime
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_futime($loop, $fd, int $utime, int $atime, callable $callback)
{
}

/**
 * Async chmod.
 *
 * @param UVLoop $loop uv loop handle.
 * @param string $path
 * @param int $mode
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_chmod($loop, string $path, int $mode, callable $callback)
{
}

/**
 * Async fchmod.
 *
 * @param UVLoop $loop uv loop handle.
 * @param resource $fd
 * @param int $mode
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_fchmod($loop, $fd, int $mode, callable $callback)
{
}

/**
 * Async chown.
 *
 * @param UVLoop $loop uv loop handle.
 * @param string $path
 * @param int $uid
 * @param int $gid
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_chown($loop, string $path, int $uid, int $gid, callable $callback)
{
}

/**
 * Async fchown.
 *
 * @param UVLoop $loop uv loop handle.
 * @param resource $fd
 * @param int $uid
 * @param int $gid
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_fchown($loop, $fd, int $uid, int $gid, callable $callback)
{
}

/**
 * Async link.
 *
 * @param UVLoop $loop uv loop handle.
 * @param string $from
 * @param string $to
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_link($loop, string $from, string $to, callable $callback)
{
}

/**
 * Async symlink.
 *
 * @param UVLoop $loop uv loop handle.
 * @param string $from
 * @param string $to
 * @param int $flags
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_symlink($loop, string $from, string $to, int $flags, callable $callback)
{
}

/**
 * Async readlink.
 *
 * @param UVLoop $loop uv loop handle
 * @param string $path
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_readlink($loop, string $path, callable $callback)
{
}

/**
 * Async stat.
 *
 * @param UVLoop $loop uv loop handle
 * @param string $path
 * @param callable $callback this callback parameter expects (resource $stream, array $stat)
 *
 * @return void
 */
function uv_fs_stat($loop, string $path, callable $callback)
{
}

/**
 * Async lstat.
 *
 * @param UVLoop $loop uv loop handle
 * @param string $path
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_lstat($loop, string $path, callable $callback)
{
}

/**
 * Async fstat.
 *
 * @param UVLoop $loop uv loop handle.
 * @param resource $fd
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_fstat($loop, $fd, callable $callback)
{
}

/**
 * Async readdir.
 *
 * @param UVLoop $loop  uv loop handle
 * @param string $path
 * @param int $flags
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_readdir($loop, string $path, int $flags, callable $callback)
{
}

/**
 * Async sendfile.
 *
 * @param UVLoop $loop uv loop handle
 * @param resource $in_fd
 * @param resource $out_fd
 * @param int $offset
 * @param int $length
 * @param callable $callback
 */
function uv_fs_sendfile($loop, $in_fd, $out_fd, int $offset, int $length, callable $callback)
{
}

/**
 * Initialize fs event.
 *
 * @param UVLoop $loop uv loop handle
 * @param string $path
 * @param callable $callback
 * @param int $flags
 *
 * @return UVFsEvent
 */
function uv_fs_event_init($loop, string $path, callable $callback, int $flags = 0)
{
}

/**
 * Initialize TTY. you have to open tty your hand.
 *
 * @param UVLoop $loop uv loop handle.
 * @param resource $fd
 * @param int $readable
 *
 * @return UVTty
 */
function uv_tty_init($loop, $fd, int $readable)
{
}

/**
 * @param UVTty $tty
 * @param int &$width
 * @param int &$height
 *
 * @return int
 */
function uv_tty_get_winsize($tty, int &$width, int &$height): int
{
}

/**
 * @param UVTty $tty
 * @param int $mode
 *
 * @return int
 */
function uv_tty_set_mode($tty, int $mode): int
{
}

/**
 * @return void
 */
function uv_tty_reset_mode()
{
}

/**
 * @param UVTcp $uv_sockaddr
 *
 * @return string
 */
function uv_tcp_getsockname($uv_sockaddr): string
{
}

/**
 * @param UVTcp $uv_sockaddr
 *
 * @return string
 */
function uv_tcp_getpeername($uv_sockaddr): string
{
}

/**
 * @param UVUdp $uv_sockaddr
 *
 * @return string
 */
function uv_udp_getsockname($uv_sockaddr): string
{
}

/**
 * @return int
 */
function uv_resident_set_memory(): int
{
}

/**
 * @param UVSockAddr $address
 *
 * @return string
 */
function uv_ip4_name($address): string
{
}

/**
 * @param UVSockAddr $address
 *
 * @return string
 */
function uv_ip6_name($address): string
{
}

/**
 * Initialize poll.
 *
 * @param UVLoop $uv_loop uv_loop.
 * @param resource $fd this expects long fd, PHP's stream or PHP's socket resource.
 *
 * @return UVPoll uv handle which initialized poll.
 */
function uv_poll_init($uv_loop, $fd)
{
}

/**
 * Start polling.
 *
 * @param UVPoll $handle uv poll handle.
 * @param int $events UV::READBLE and UV::WRITABLE flags.
 * @param callable $callback this callback parameter expects (UVPoll $poll, long $status, long $events, mixed $connection). the connection parameter passes uv_poll_init'd fd.
 *
 * @return void
 */
function uv_poll_start($handle, int $events, callable $callback)
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
