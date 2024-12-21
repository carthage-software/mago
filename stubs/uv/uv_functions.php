<?php

// Stubs for ext-uv: https://github.com/amphp/php-uv

/**
 * Decrement reference.
 *
 * @param UV $uv_t uv handle.
 *
 * @return void
 */
function uv_unref($uv_t) {}

/**
 * Get last error code.
 *
 * @param UVLoop|null $uv_loop uv loop handle.
 * @return int
 */
function uv_last_error($uv_loop = null): int {}

/**
 * Get error code name.
 *
 * @param int $error_code libuv error code.
 * @return string
 */
function uv_err_name(int $error_code): string {}

/**
 * Get error message.
 *
 * @param int $error_code libuv error code
 * @return string
 */
function uv_strerror(int $error_code): string {}

/**
 * @param UVLoop $uv_loop uv loop handle.
 *
 * @return void
 */
function uv_update_time($uv_loop) {}

/**
 * Increment reference count.
 *
 * @param UV $uv_handle
 *
 * @return void
 */
function uv_ref($uv_handle) {}

/**
 * Run event loop.
 *
 * @param UVLoop|null $uv_loop
 * @param int $run_mode Run flags.
 *
 * @return void
 */
function uv_run($uv_loop = null, $run_mode = UV::RUN_DEFAULT) {}

/**
 * @param UVLoop|null $uv_loop
 *
 * @return void
 */
function uv_run_once($uv_loop = null) {}

/**
 * Delete specified loop.
 *
 * @param UVLoop $uv_loop
 *
 * @return void
 */
function uv_loop_delete($uv_loop) {}

/**
 * @return int
 */
function uv_now(): int {}

/**
 * Binds a name to a socket.
 *
 * @param UVTcp $uv_tcp
 * @param UVSockAddr $uv_sockaddr
 *
 * @return void
 */
function uv_tcp_bind($uv_tcp, $uv_sockaddr) {}

/**
 * Binds a name to a socket.
 *
 * @param UVTcp $uv_tcp
 * @param UVSockAddr $uv_sockaddr
 *
 * @return void
 */
function uv_tcp_bind6($uv_tcp, $uv_sockaddr) {}

/**
 * Send buffer to specified uv stream.
 *
 * @param UVStream $handle uv stream (uv_tcp, uv_udp, uv_pipe ...etc.).
 * @param string $data buffer.
 * @param callable $callback callable variables. This callback expects (UVStream $handle, long $status).
 *
 * @return void
 */
function uv_write($handle, string $data, callable $callback) {}

/**
 * @param UVStream $handle
 * @param string $data
 * @param UVTcp|UvPipe $send
 * @param callable $callback
 *
 * @return void
 */
function uv_write2($handle, string $data, $send, callable $callback) {}

/**
 * Set Nagel's flags for specified tcp stream.
 *
 * @param UVTcp $handle libuv tcp handle.
 * @param bool $enable true means enabled. false means disabled.
 */
function uv_tcp_nodelay($handle, bool $enable) {}

/**
 * Accepts a connection on a socket.
 *
 * @param UVTcp|UVPipe $server uv_tcp or uv_pipe server.
 * @param UVTcp|UVPipe $client uv_tcp or uv_pipe client.
 *
 * @return void
 */
function uv_accept($server, $client) {}

/**
 * Shutdown uv handle.
 *
 * @param UVStream $handle uv stream (uv_tcp, uv_udp, uv_pipe ...etc.).
 * @param callable $callback callable variables. this callback expects (UVStream $handle, long $status).
 *
 * @return void
 */
function uv_shutdown($handle, callable $callback) {}

/**
 * Close uv handle.
 *
 * @param UV $handle uv handle (uv_tcp, uv_udp, uv_pipe ...etc.).
 * @param ?callable $callback callable variables. this callback expects (UV $handle, long $status).
 *
 * @return void
 */
function uv_close($handle, callable $callback = null) {}

/**
 * Starts read callback for uv streams.
 *
 * Note: You have to handle errors correctly. otherwise this will leak.
 * Note: if you want to use PHP's stream or socket resource. see uv_fs_poll_init and uv_fs_read.
 *
 * @param UVStream $handle uv stream (uv_tcp, uv_udp, uv_pipe ...etc.)
 * @param callable $callback callable variables. this callback parameter expects (UVStream $handle, long $nread, string buffer).
 */
function uv_read_start($handle, callable $callback) {}

/**
 * Stop read callback.
 *
 * @param UVStream $handle uv stream handle which started uv_read.
 *
 * @return void
 */
function uv_read_stop($handle) {}

/**
 * Create a ipv4 sockaddr.
 *
 * @param string $ipv4_addr ipv4 address
 * @param int $port port number.
 *
 * @return UVSockAddrIPv4
 */
function uv_ip4_addr(string $ipv4_addr, int $port) {}

/**
 * Create a ipv6 sockaddr.
 *
 * @param string $ipv6_addr ipv6 address.
 * @param int $port port number.
 *
 * @return UVSockAddrIPv6
 */
function uv_ip6_addr(string $ipv6_addr, int $port) {}

/**
 * Listens for a connection on a uv handle.
 *
 * @param UVTcp|UVPipe $handle uv handle (tcp, udp and pipe).
 * @param int $backlog backlog.
 * @param callable $callback this callback parameter expects (UVTcp|UVPipe $connection, long $status).
 *
 * @return void
 */
function uv_listen($handle, int $backlog, callable $callback) {}

/**
 * Connect to specified ip address and port.
 *
 * @param UVTcp $handle requires uv_tcp_init().
 * @param UVSockAddr $ipv4_addr requires uv_sockaddr.
 * @param callable $callback callable variables. This callback expects (UVTcp $tcp_handle, $status).
 *
 * @return void
 */
function uv_tcp_connect($handle, $ipv4_addr, callable $callback) {}

/**
 * Connect to specified ip address and port.
 *
 * @param UVTcp $handle requires uv_tcp_init().
 * @param UVSockAddrIPv6 $ipv6_addr requires uv_sockaddr.
 * @param callable $callback callable variables. This callback expects (UVTcp $tcp_handle, $status).
 *
 * @return void
 */
function uv_tcp_connect6($handle, $ipv6_addr, callable $callback) {}

/**
 * Initialize timer handle.
 *
 * @param UVLoop|null $loop uv_loop handle.
 *
 * @return UVTimer
 */
function uv_timer_init($loop = null) {}

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
function uv_timer_start($timer, int $timeout, int $repeat, callable $callback) {}

/**
 * stop specified timer.
 *
 * @param UVTimer $timer uv timer.
 *
 * @return int
 */
function uv_timer_stop($timer): int {}

/**
 * Restart timer.
 *
 * @param UVTimer $timer uv_timer.
 *
 * @return void
 */
function uv_timer_again($timer) {}

/**
 * Set repeat count.
 *
 * @param UVTimer $timer uv_timer.
 * @param int $repeat repeat count.
 *
 * @return void
 */
function uv_timer_set_repeat($timer, int $repeat) {}

/**
 * Returns repeat interval.
 *
 * @param UVTimer $timer uv_timer.
 *
 * @return int
 */
function uv_timer_get_repeat($timer): int {}

/**
 * Initialize uv idle handle.
 *
 * @param UVLoop $loop uv_loop handle.
 *
 * @return UVIdle initialized idle handle.
 */
function uv_idle_init($loop = null) {}

/**
 * start idle callback.
 *
 * @param UVIdle $idle uv_idle.
 * @param callable $callback idle callback.
 *
 * @return void
 */
function uv_idle_start($idle, callable $callback) {}

/**
 * Stop idle callback.
 *
 * @param UVIdle $idle uv_idle.
 *
 * @return void
 */
function uv_idle_stop($idle) {}

/**
 * @param UVLoop $loop
 * @param callable $callback
 * @param string $node
 * @param string $service
 * @param array $hints
 *
 * @return void
 */
function uv_getaddrinfo($loop, callable $callback, string $node, string $service, array $hints) {}

/**
 * Create a tcp socket.
 *
 * @param UVLoop|null $loop loop or null. if not specified loop then use uv_default_loop.
 *
 * @return UVTcp uv handle which initialized for tcp.
 */
function uv_tcp_init($loop = null) {}

/**
 * Return default loop handle.
 *
 * @return UVLoop
 */
function uv_default_loop() {}

/**
 * Create a new loop handle.
 *
 * @return UVLoop
 */
function uv_loop_new() {}

/**
 * Create a udp socket.
 *
 * @param UVLoop|null $loop loop or null. if not specified loop then use uv_default_loop.
 *
 * @return UVUdp uv handle which initialized for udp.
 */
function uv_udp_init($loop = null) {}

/**
 * Listens for a connection on a uv udp handle.
 *
 * @param UVUdp $resource uv udp handle.
 * @param UVSockAddr $address uv sockaddr(ipv4).
 * @param int $flags unused.
 *
 * @return void
 */
function uv_udp_bind($resource, $address, int $flags) {}

/**
 * Listens for a connection on a uv udp handle.
 *
 * @param UVUdp $resource uv udp handle.
 * @param UVSockAddr $address uv sockaddr(ipv6).
 * @param int $flags Should be 0 or UV::UDP_IPV6ONLY.
 *
 * @return void
 */
function uv_udp_bind6($resource, $address, int $flags) {}

/**
 * Start receive callback.
 *
 * @param UVUdp $handle uv udp handle.
 * @param callable $callback this callback parameter expects (UVUdp $stream, long $nread, string $buffer)..
 *
 * @return void
 */
function uv_udp_recv_start($handle, callable $callback) {}

/**
 * Stop receive callback.
 *
 * @param UVUdp $handle
 *
 * @return void
 */
function uv_udp_recv_stop($handle) {}

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
function uv_udp_set_membership($handle, string $multicast_addr, string $interface_addr, int $membership): int {}

/**
 * Set multicast loop.
 *
 * @param UVUdp $handle uv udp handle.
 * @param int $enabled
 *
 * @return void
 */
function uv_udp_set_multicast_loop($handle, int $enabled) {}

/**
 * Set multicast ttl.
 *
 * @param UVUdp $handle uv udp handle.
 * @param int $ttl multicast ttl.
 *
 * @return void
 */
function uv_udp_set_multicast_ttl($handle, int $ttl) {}

/**
 * Set udp broadcast.
 *
 * @param UVUdp $handle uv udp handle.
 * @param bool $enabled
 *
 * @return void
 */
function uv_udp_set_broadcast($handle, bool $enabled) {}

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
function uv_udp_send($handle, string $data, $uv_addr, callable $callback) {}

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
function uv_udp_send6($handle, string $data, $uv_addr6, callable $callback) {}

/**
 * @param UV $handle
 *
 * @return bool
 */
function uv_is_active($handle): bool {}

/**
 * @param UV $handle
 *
 * @return bool
 */
function uv_is_closing($handle): bool {}

/**
 * @param UVStream $handle
 *
 * @return bool
 */
function uv_is_readable($handle): bool {}

/**
 * @param UVStream $handle
 *
 * @return bool
 */
function uv_is_writable($handle): bool {}

/**
 * @param UVLoop $loop
 * @param callable $closure
 * @param array|null $opaque
 *
 * @return bool
 */
function uv_walk($loop, callable $closure, array $opaque = null): bool {}

/**
 * @param resource $uv
 *
 * @return int
 */
function uv_guess_handle($uv): int {}

/**
 * Initialize pipe.
 *
 * @param UVLoop $loop uv_loop handle.
 * @param int $ipc when this pipe use for ipc, please set true otherwise false.
 *
 * @return UVPipe
 */
function uv_pipe_init($loop, int $ipc) {}

/**
 * Open a pipe.
 *
 * @param UVPipe $handle uv pipe handle.
 * @param int $pipe dunnno. maybe file descriptor.
 *
 * @return void
 */
function uv_pipe_open($handle, int $pipe) {}

/**
 * Create a named pipe.
 *
 * @param UVPipe $handle uv pipe handle.
 * @param string $name dunnno. maybe file descriptor.
 *
 * @return int
 */
function uv_pipe_bind($handle, string $name): int {}

/**
 * Connect to named pipe.
 *
 * @param UVPipe $handle uv pipe handle.
 * @param string $path named pipe path.
 * @param callable $callback this callback parameter expects (UVPipe $pipe, long $status).
 *
 * @return void
 */
function uv_pipe_connect($handle, string $path, callable $callback) {}

/**
 * @param UVPipe $handle
 * @param void $count
 *
 * @return void
 */
function uv_pipe_pending_instances($handle, $count) {}

/**
 * Returns current loadaverage.
 *
 * Note: returns array on windows box. (does not support load average on windows).
 *
 * @return array
 */
function uv_loadavg(): array {}

/**
 * Returns current uptime.
 *
 * @return float
 */
function uv_uptime(): float {}

/**
 * Returns current free memory size.
 *
 * @return int
 */
function uv_get_free_memory(): int {}

/**
 * Returns total memory size.
 *
 * @return int
 */
function uv_get_total_memory(): int {}

/**
 * @return int
 */
function uv_hrtime(): int {}

/**
 * Returns current exepath. basically this will returns current php path.
 *
 * @return string
 */
function uv_exepath(): string {}

/**
 * Returns current cpu informations.
 *
 * @return array
 */
function uv_cpu_info(): array {}

/**
 * @return array
 */
function uv_interface_addresses(): array {}

/**
 * @param UV|resource|long|null $fd
 * @param int $flags
 *
 * @return UVStdio
 */
function uv_stdio_new($fd, int $flags) {}

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
function uv_spawn($loop, string $command, array $args, array $stdio, string $cwd, array $env = [], ?callable $callback = null, ?int $flags = null, ?array $options = null) {}

/**
 * Send signal to specified uv process.
 *
 * @param UVProcess $handle uv process handle.
 * @param int $signal
 *
 * @return void
 */
function uv_process_kill($handle, int $signal) {}

/**
 * Send signal to specified pid.
 *
 * @param int $pid process id.
 * @param int $signal
 */
function uv_kill(int $pid, int $signal) {}

/**
 * Change working directory.
 *
 * @param string $directory
 * @return bool
 */
function uv_chdir(string $directory): bool {}

/**
 * Initialize rwlock.
 *
 * @return UVLock returns uv rwlock.
 */
function uv_rwlock_init() {}

/**
 * Set read lock.
 *
 * @param UVLock $handle uv lock handle.
 */
function uv_rwlock_rdlock($handle) {}

/**
 * @param UVLock $handle
 *
 * @return bool
 */
function uv_rwlock_tryrdlock($handle): bool {}

/**
 * Unlock read lock.
 *
 * @param UVLock $handle uv lock handle.
 *
 * @return void
 */
function uv_rwlock_rdunlock($handle) {}

/**
 * Set write lock.
 *
 * @param UVLock $handle uv lock handle.
 *
 * @return void
 */
function uv_rwlock_wrlock($handle) {}

/**
 * @param UVLock $handle uv lock handle.
 */
function uv_rwlock_trywrlock($handle) {}

/**
 * Unlock write lock.
 *
 * @param UVLock $handle uv lock handle.
 */
function uv_rwlock_wrunlock($handle) {}

/**
 * Initialize mutex.
 *
 * @return UVLock uv mutex
 */
function uv_mutex_init() {}

/**
 * Lock mutex.
 *
 * @param UVLock $lock uv lock handle.
 *
 * @return void
 */
function uv_mutex_lock($lock) {}

/**
 * @param UVLock $lock
 *
 * @return bool
 */
function uv_mutex_trylock($lock): bool {}

/**
 * Initialize semaphore.
 *
 * @param int $value
 * @return UVLock
 */
function uv_sem_init(int $value) {}

/**
 * Post semaphore.
 *
 * @param UVLock $sem uv lock handle.
 *
 * @return void
 */
function uv_sem_post($sem) {}

/**
 * @param UVLock $sem
 *
 * @return void
 */
function uv_sem_wait($sem) {}

/**
 * @param UVLock $sem
 *
 * @return void
 */
function uv_sem_trywait($sem) {}

/**
 * Initialize prepare.
 *
 * @param UVLoop $loop uv loop handle.
 *
 * @return UVPrepare
 */
function uv_prepare_init($loop) {}

/**
 * Setup prepare loop callback. (pre loop callback)
 *
 * @param UVPrepare $handle uv prepare handle.
 * @param callable $callback this callback parameter expects (UVPrepare $prepare, long $status).
 *
 * @return void
 */
function uv_prepare_start($handle, callable $callback) {}

/**
 * Stop prepare callback.
 *
 * @param UVPrepare $handle uv prepare handle.
 *
 * @return void
 */
function uv_prepare_stop($handle) {}

/**
 * Setup check.
 *
 * @param UVLoop $loop uv loop handle
 *
 * @return UVCheck
 */
function uv_check_init($loop) {}

/**
 * Stats check loop callback. (after loop callback)
 *
 * @param UVCheck $handle uv check handle.
 * @param callable $callback this callback parameter expects (UVCheck $check, long $status).
 *
 * @return void
 */
function uv_check_start($handle, callable $callback) {}

/**
 * Stop check callback.
 *
 * @param UVCheck $handle uv check handle.
 *
 * @return void
 */
function uv_check_stop($handle) {}

/**
 * Setup async callback.
 *
 * @param UVLoop $loop uv loop
 * @param callable $callback
 *
 * @return UVAsync uv async handle.
 */
function uv_async_init($loop, callable $callback) {}

/**
 * Send async callback immediately.
 *
 * @param UVAsync $handle uv async handle.
 *
 * @return void
 */
function uv_async_send($handle) {}

/**
 * Execute callbacks in another thread (requires Thread Safe enabled PHP).
 *
 * @param UVLoop $loop
 * @param callable $callback
 * @param callable $after_callback
 *
 * @return void
 */
function uv_queue_work($loop, callable $callback, callable $after_callback) {}

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
function uv_fs_open($loop, string $path, int $flag, int $mode, callable $callback) {}

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
function uv_fs_read($loop, $fd, int $offset, int $length, callable $callback) {}

/**
 * Close specified file descriptor.
 *
 * @param UVLoop $loop uv_loop handle.
 * @param resource $fd file descriptor. this expects long $fd, resource $php_stream or resource $php_socket.
 * @param callable $callback this callback parameter expects (resource $stream)
 *
 * @return void
 */
function uv_fs_close($loop, $fd, callable $callback) {}

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
function uv_fs_write($loop, $fd, string $buffer, int $offset, callable $callback) {}

/**
 * Async fsync.
 *
 * @param UVLoop $loop uv loop handle.
 * @param resource $fd
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_fsync($loop, $fd, callable $callback) {}

/**
 * Async fdatasync.
 *
 * @param UVLoop $loop uv loop handle.
 * @param resource $fd
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_fdatasync($loop, $fd, callable $callback) {}

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
function uv_fs_ftruncate($loop, $fd, int $offset, callable $callback) {}

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
function uv_fs_mkdir($loop, string $path, int $mode, callable $callback) {}

/**
 * Async rmdir.
 *
 * @param UVLoop $loop uv loop handle
 * @param string $path
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_rmdir($loop, string $path, callable $callback) {}

/**
 * Async unlink.
 *
 * @param UVLoop $loop uv loop handle
 * @param string $path
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_unlink($loop, string $path, callable $callback) {}

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
function uv_fs_rename($loop, string $from, string $to, callable $callback) {}

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
function uv_fs_utime($loop, string $path, int $utime, int $atime, callable $callback) {}

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
function uv_fs_futime($loop, $fd, int $utime, int $atime, callable $callback) {}

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
function uv_fs_chmod($loop, string $path, int $mode, callable $callback) {}

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
function uv_fs_fchmod($loop, $fd, int $mode, callable $callback) {}

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
function uv_fs_chown($loop, string $path, int $uid, int $gid, callable $callback) {}

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
function uv_fs_fchown($loop, $fd, int $uid, int $gid, callable $callback) {}

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
function uv_fs_link($loop, string $from, string $to, callable $callback) {}

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
function uv_fs_symlink($loop, string $from, string $to, int $flags, callable $callback) {}

/**
 * Async readlink.
 *
 * @param UVLoop $loop uv loop handle
 * @param string $path
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_readlink($loop, string $path, callable $callback) {}

/**
 * Async stat.
 *
 * @param UVLoop $loop uv loop handle
 * @param string $path
 * @param callable $callback this callback parameter expects (resource $stream, array $stat)
 *
 * @return void
 */
function uv_fs_stat($loop, string $path, callable $callback) {}

/**
 * Async lstat.
 *
 * @param UVLoop $loop uv loop handle
 * @param string $path
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_lstat($loop, string $path, callable $callback) {}

/**
 * Async fstat.
 *
 * @param UVLoop $loop uv loop handle.
 * @param resource $fd
 * @param callable $callback
 *
 * @return void
 */
function uv_fs_fstat($loop, $fd, callable $callback) {}

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
function uv_fs_readdir($loop, string $path, int $flags, callable $callback) {}

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
function uv_fs_sendfile($loop, $in_fd, $out_fd, int $offset, int $length, callable $callback) {}

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
function uv_fs_event_init($loop, string $path, callable $callback, int $flags = 0) {}

/**
 * Initialize TTY. you have to open tty your hand.
 *
 * @param UVLoop $loop uv loop handle.
 * @param resource $fd
 * @param int $readable
 *
 * @return UVTty
 */
function uv_tty_init($loop, $fd, int $readable) {}

/**
 * @param UVTty $tty
 * @param int &$width
 * @param int &$height
 *
 * @return int
 */
function uv_tty_get_winsize($tty, int &$width, int &$height): int {}

/**
 * @param UVTty $tty
 * @param int $mode
 *
 * @return int
 */
function uv_tty_set_mode($tty, int $mode): int {}

/**
 * @return void
 */
function uv_tty_reset_mode() {}

/**
 * @param UVTcp $uv_sockaddr
 *
 * @return string
 */
function uv_tcp_getsockname($uv_sockaddr): string {}

/**
 * @param UVTcp $uv_sockaddr
 *
 * @return string
 */
function uv_tcp_getpeername($uv_sockaddr): string {}

/**
 * @param UVUdp $uv_sockaddr
 *
 * @return string
 */
function uv_udp_getsockname($uv_sockaddr): string {}

/**
 * @return int
 */
function uv_resident_set_memory(): int {}

/**
 * @param UVSockAddr $address
 *
 * @return string
 */
function uv_ip4_name($address): string {}

/**
 * @param UVSockAddr $address
 *
 * @return string
 */
function uv_ip6_name($address): string {}

/**
 * Initialize poll.
 *
 * @param UVLoop $uv_loop uv_loop.
 * @param resource $fd this expects long fd, PHP's stream or PHP's socket resource.
 *
 * @return UVPoll uv handle which initialized poll.
 */
function uv_poll_init($uv_loop, $fd) {}

/**
 * Start polling.
 *
 * @param UVPoll $handle uv poll handle.
 * @param int $events UV::READBLE and UV::WRITABLE flags.
 * @param callable $callback this callback parameter expects (UVPoll $poll, long $status, long $events, mixed $connection). the connection parameter passes uv_poll_init'd fd.
 *
 * @return void
 */
function uv_poll_start($handle, int $events, callable $callback) {}

/**
 * @param UVPoll $poll
 *
 * @return void
 */
function uv_poll_stop($poll) {}

/**
 * @param UVLoop|null $uv_loop
 *
 * @return UVFsPoll
 */
function uv_fs_poll_init($uv_loop = null) {}

/**
 * @param UVFsPoll $handle
 * @param $callback
 * @param string $path
 * @param int $interval
 *
 * @return UV
 */
function uv_fs_poll_start($handle, $callback, string $path, int $interval) {}

/**
 * @param UVFsPoll $poll
 *
 * @return void
 */
function uv_fs_poll_stop($poll) {}

/**
 * @param UVLoop $uv_loop uv loop handle
 *
 * @return void
 */
function uv_stop($uv_loop) {}

/**
 * @param UVSignal $sig_handle
 *
 * @return int
 */
function uv_signal_stop($sig_handle): int {}
