<?php

namespace {
    const WNOHANG = 1;

    const WUNTRACED = 2;

    const WCONTINUED = 8;

    const SIG_IGN = 1;

    const SIG_DFL = 0;

    const SIG_ERR = -1;

    const SIGHUP = 1;

    const SIGINT = 2;

    const SIGQUIT = 3;

    const SIGILL = 4;

    const SIGTRAP = 5;

    const SIGABRT = 6;

    const SIGIOT = 6;

    const SIGBUS = 7;

    const SIGFPE = 8;

    const SIGKILL = 9;

    const SIGUSR1 = 10;

    const SIGSEGV = 11;

    const SIGUSR2 = 12;

    const SIGPIPE = 13;

    const SIGALRM = 14;

    const SIGTERM = 15;

    const SIGSTKFLT = 16;

    const SIGCLD = 17;

    const SIGCHLD = 17;

    const SIGCONT = 18;

    const SIGSTOP = 19;

    const SIGTSTP = 20;

    const SIGTTIN = 21;

    const SIGTTOU = 22;

    const SIGURG = 23;

    const SIGXCPU = 24;

    const SIGXFSZ = 25;

    const SIGVTALRM = 26;

    const SIGPROF = 27;

    const SIGWINCH = 28;

    const SIGPOLL = 29;

    const SIGIO = 29;

    const SIGPWR = 30;

    const SIGSYS = 31;

    const SIGBABY = 31;

    const PRIO_PGRP = 1;

    const PRIO_USER = 2;

    const PRIO_PROCESS = 0;

    const SIG_BLOCK = 0;

    const SIG_UNBLOCK = 1;

    const SIG_SETMASK = 2;

    const SIGRTMIN = 35;

    const SIGRTMAX = 64;

    const SI_USER = 0;

    const SI_KERNEL = 128;

    const SI_QUEUE = -1;

    const SI_TIMER = -2;

    const SI_MESGQ = -3;

    const SI_ASYNCIO = -4;

    const SI_SIGIO = -5;

    const SI_TKILL = -6;

    const CLD_EXITED = 1;

    const CLD_KILLED = 2;

    const CLD_DUMPED = 3;

    const CLD_TRAPPED = 4;

    const CLD_STOPPED = 5;

    const CLD_CONTINUED = 6;

    const TRAP_BRKPT = 1;

    const TRAP_TRACE = 2;

    const POLL_IN = 1;

    const POLL_OUT = 2;

    const POLL_MSG = 3;

    const POLL_ERR = 4;

    const POLL_PRI = 5;

    const POLL_HUP = 6;

    const ILL_ILLOPC = 1;

    const ILL_ILLOPN = 2;

    const ILL_ILLADR = 3;

    const ILL_ILLTRP = 4;

    const ILL_PRVOPC = 5;

    const ILL_PRVREG = 6;

    const ILL_COPROC = 7;

    const ILL_BADSTK = 8;

    const FPE_INTDIV = 1;

    const FPE_INTOVF = 2;

    const FPE_FLTDIV = 3;

    const FPE_FLTOVF = 4;

    const FPE_FLTUND = 5;

    const FPE_FLTRES = 6;

    const FPE_FLTINV = 7;

    const FPE_FLTSUB = 8;

    const SEGV_MAPERR = 1;

    const SEGV_ACCERR = 2;

    const BUS_ADRALN = 1;

    const BUS_ADRERR = 2;

    const BUS_OBJERR = 3;

    const PCNTL_EINTR = 4;

    const PCNTL_ECHILD = 10;

    const PCNTL_EINVAL = 22;

    const PCNTL_EAGAIN = 11;

    const PCNTL_ESRCH = 3;

    const PCNTL_EACCES = 13;

    const PCNTL_EPERM = 1;

    const PCNTL_ENOMEM = 12;

    const PCNTL_E2BIG = 7;

    const PCNTL_EFAULT = 14;

    const PCNTL_EIO = 5;

    const PCNTL_EISDIR = 21;

    const PCNTL_ELIBBAD = 80;

    const PCNTL_ELOOP = 40;

    const PCNTL_EMFILE = 24;

    const PCNTL_ENAMETOOLONG = 36;

    const PCNTL_ENFILE = 23;

    const PCNTL_ENOENT = 2;

    const PCNTL_ENOEXEC = 8;

    const PCNTL_ENOTDIR = 20;

    const PCNTL_ETXTBSY = 26;

    const PCNTL_ENOSPC = 28;

    const PCNTL_EUSERS = 87;

    const CLONE_NEWNS = 131072;

    const CLONE_NEWIPC = 134217728;

    const CLONE_NEWUTS = 67108864;

    const CLONE_NEWNET = 1073741824;

    const CLONE_NEWPID = 536870912;

    const CLONE_NEWUSER = 268435456;

    const CLONE_NEWCGROUP = 33554432;

    const P_ALL = 0;

    const WEXITED = 4;

    const WSTOPPED = 2;

    const WNOWAIT = 16777216;

    const P_PID = 1;

    const P_PGID = 2;

    const P_PIDFD = 3;

    function pcntl_fork(): int {}

    /**
     * @param-out array $resource_usage
     */
    function pcntl_waitpid(int $process_id, &$status, int $flags = 0, &$resource_usage = []): int {}

    /**
     * @param-out int $status
     * @param-out array $resource_usage
     */
    function pcntl_wait(&$status, int $flags = 0, &$resource_usage = []): int {}

    /**
     * @param callable|int $handler
     */
    function pcntl_signal(int $signal, $handler, bool $restart_syscalls = true): bool {}

    function pcntl_signal_dispatch(): bool {}

    function pcntl_wifexited(int $status): bool {}

    function pcntl_wifstopped(int $status): bool {}

    function pcntl_wifsignaled(int $status): bool {}

    function pcntl_wexitstatus(int $status): int|false {}

    function pcntl_wifcontinued(int $status): bool {}

    function pcntl_wtermsig(int $status): int|false {}

    function pcntl_wstopsig(int $status): int|false {}

    function pcntl_exec(string $path, array $args = [], array $env_vars = []): bool {}

    function pcntl_alarm(int $seconds): int {}

    function pcntl_get_last_error(): int {}

    function pcntl_errno(): int {}

    function pcntl_strerror(int $error_code): string {}

    function pcntl_getpriority(?int $process_id, int $mode = PRIO_PROCESS): int|false {}

    function pcntl_setpriority(int $priority, ?int $process_id, int $mode = PRIO_PROCESS): bool {}

    /**
     * @param-out array $old_signals
     */
    function pcntl_sigprocmask(int $mode, array $signals, &$old_signals): bool {}

    /**
     * @param-out array $info
     */
    function pcntl_sigwaitinfo(array $signals, &$info = []): int|false {}

    /**
     * @param-out array $info
     */
    function pcntl_sigtimedwait(array $signals, &$info = [], int $seconds = 0, int $nanoseconds = 0): int|false {}

    function pcntl_async_signals(?bool $enable = null): bool {}

    /**
     * @return bool|resource
     */
    function pcntl_signal_get_handler(int $signal) {}

    function pcntl_unshare(int $flags): bool {}

    function pcntl_waitid(int $idtype = P_ALL, ?int $id = null, &$info = [], int $flags = WEXITED): bool {}

    function pcntl_getcpuaffinity(?int $process_id = null): array|false {}

    function pcntl_setcpuaffinity(?int $process_id = null, array $cpu_ids = []): bool {}

    function pcntl_getcpu(): int {}
}

namespace Pcntl {
    enum QosClass
    {
        case Background;
        case Utility;
        case Default;
        case UserInitiated;
        case UserInteractive;
    }
}
