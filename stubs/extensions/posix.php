<?php

function posix_kill(int $process_id, int $signal): bool
{
}

function posix_getpid(): int
{
}

function posix_getppid(): int
{
}

function posix_getuid(): int
{
}

function posix_setuid(int $user_id): bool
{
}

function posix_geteuid(): int
{
}

function posix_seteuid(int $user_id): bool
{
}

function posix_setrlimit(int $resource, int $soft_limit, int $hard_limit): bool
{
}

function posix_getgid(): int
{
}

function posix_setgid(int $group_id): bool
{
}

function posix_getegid(): int
{
}

function posix_setegid(int $group_id): bool
{
}

/**
 * @return list<int>|false
 */
function posix_getgroups(): array|false
{
}

function posix_getlogin(): string|false
{
}

function posix_getpgrp(): int
{
}

function posix_setsid(): int
{
}

function posix_setpgid(int $process_id, int $process_group_id): bool
{
}

function posix_getpgid(int $process_id): int|false
{
}

function posix_getsid(int $process_id): int|false
{
}

/**
 * @return array{
 *   'sysname': string,
 *   'nodename': string,
 *   'release': string,
 *   'version': string,
 *   'machine': string,
 *   'domainname': string
 * }|false
 */
function posix_uname(): array|false
{
}

/**
 * @return array{
 *   'ticks': int,
 *   'utime': int,
 *   'stime': int,
 *   'cutime': int,
 *   'cstime': int
 * }|false
 */
function posix_times(): array|false
{
}

function posix_ctermid(): string|false
{
}

/**
 * @param resource|int $file_descriptor
 */
function posix_ttyname($file_descriptor): string|false
{
}

/**
 * @param resource|int $file_descriptor
 */
function posix_isatty($file_descriptor): bool
{
}

function posix_getcwd(): string|false
{
}

function posix_mkfifo(string $filename, int $permissions): bool
{
}

function posix_mknod(string $filename, int $flags, int $major = 0, int $minor = 0): bool
{
}

function posix_access(string $filename, int $flags = POSIX_F_OK): bool
{
}

function posix_getgrnam(string $name): array|false
{
}

function posix_getgrgid(int $group_id): array|false
{
}

/**
 * @return array{
 *   'name': string,
 *   'passwd': string,
 *   'uid': int,
 *   'gid': int,
 *   'gecos': string,
 *   'dir': string,
 *   'shell': string
 * }|false
 */
function posix_getpwnam(string $username): array|false
{
}

/**
 * @return array{
 *   'name': string,
 *   'passwd': string,
 *   'uid': int,
 *   'gid': int,
 *   'gecos': string,
 *   'dir': string,
 *   'shell': string,
 * }|false
 */
function posix_getpwuid(int $user_id): array|false
{
}

function posix_getrlimit(null|int $resource = null): array|false
{
}

function posix_get_last_error(): int
{
}

function posix_errno(): int
{
}

function posix_strerror(int $error_code): string
{
}

function posix_initgroups(string $username, int $group_id): bool
{
}

function posix_sysconf(int $conf_id): int
{
}

function posix_eaccess(string $filename, int $flags = 0): bool
{
}

/**
 * @param resource|int $file_descriptor
 */
function posix_fpathconf($file_descriptor, int $name): int|false
{
}

function posix_pathconf(string $path, int $name): int|false
{
}

const POSIX_F_OK = 0;

const POSIX_X_OK = 1;

const POSIX_W_OK = 2;

const POSIX_R_OK = 4;

const POSIX_S_IFREG = 32768;

const POSIX_S_IFCHR = 8192;

const POSIX_S_IFBLK = 24576;

const POSIX_S_IFIFO = 4096;

const POSIX_S_IFSOCK = 49152;

const POSIX_RLIMIT_AS = 5;

const POSIX_RLIMIT_CORE = 4;

const POSIX_RLIMIT_CPU = 0;

const POSIX_RLIMIT_DATA = 2;

const POSIX_RLIMIT_FSIZE = 1;

const POSIX_RLIMIT_LOCKS = 10;

const POSIX_RLIMIT_MSGQUEUE = 12;

const POSIX_RLIMIT_NICE = 13;

const POSIX_RLIMIT_RTPRIO = 14;

const POSIX_RLIMIT_RTTIME = 15;

const POSIX_RLIMIT_SIGPENDING = 11;

const POSIX_RLIMIT_MEMLOCK = 6;

const POSIX_RLIMIT_NOFILE = 8;

const POSIX_RLIMIT_NPROC = 7;

const POSIX_RLIMIT_RSS = 5;

const POSIX_RLIMIT_STACK = 3;

const POSIX_RLIMIT_INFINITY = 9223372036854775807;

const POSIX_SC_ARG_MAX = 0;

const POSIX_SC_PAGESIZE = 30;

const POSIX_SC_NPROCESSORS_CONF = 83;

const POSIX_SC_NPROCESSORS_ONLN = 84;

const POSIX_PC_LINK_MAX = 0;

const POSIX_PC_MAX_CANON = 1;

const POSIX_PC_MAX_INPUT = 2;

const POSIX_PC_NAME_MAX = 3;

const POSIX_PC_PATH_MAX = 4;

const POSIX_PC_PIPE_BUF = 5;

const POSIX_PC_CHOWN_RESTRICTED = 6;

const POSIX_PC_NO_TRUNC = 7;

const POSIX_PC_ALLOC_SIZE_MIN = 18;

const POSIX_PC_SYMLINK_MAX = 19;

const POSIX_SC_CHILD_MAX = 1;

const POSIX_SC_CLK_TCK = 2;
