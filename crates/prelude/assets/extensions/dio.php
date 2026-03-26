<?php

/**
 * @param resource $fd
 */
function dio_close($fd): void {}

/**
 * @param resource $fd
 */
function dio_fcntl($fd, int $cmd, ...$args) {}

/**
 * @return resource|false
 */
function dio_open(string $filename, int $flags, int $mode = 0) {}

/**
 * @param resource $fd
 *
 * @return string
 */
function dio_read($fd, int $len = 1024) {}

/**
 * @param resource $fd
 *
 * @return int
 */
function dio_seek($fd, int $pos, int $whence = SEEK_SET) {}

/**
 * @param resource $fd
 *
 * @return array|null
 */
function dio_stat($fd) {}

/**
 * @param resource $fd
 *
 * @return void
 */
function dio_tcsetattr($fd, array $options) {}

/**
 * @param resource $fd
 * @param int $offset
 *
 * @return bool
 */
function dio_truncate($fd, int $offset) {}

/**
 * @param resource $fd
 *
 * @return int
 */
function dio_write($fd, string $data, int $len = 0) {}

/**
 * @return resource|null
 */
function dio_raw(string $filename, string $mode, ?array $options) {}

/**
 * @return resource|null
 */
function dio_serial(string $filename, string $mode, ?array $options) {}

const O_RDONLY = 0;

const O_WRONLY = 1;

const O_RDWR = 2;

const O_CREAT = 64;

const O_EXCL = 128;

const O_TRUNC = 512;

const O_APPEND = 1024;

const O_NONBLOCK = 2048;

const O_NDELAY = 2048;

const O_SYNC = 1052672;

const O_ASYNC = 8192;

const O_NOCTTY = 256;

const S_IRWXU = 448;

const S_IRUSR = 256;

const S_IWUSR = 128;

const S_IXUSR = 64;

const S_IRWXG = 56;

const S_IRGRP = 32;

const S_IWGRP = 16;

const S_IXGRP = 8;

const S_IRWXO = 7;

const S_IROTH = 4;

const S_IWOTH = 2;

const S_IXOTH = 1;

const F_DUPFD = 0;

const F_GETFD = 1;

const F_GETFL = 3;

const F_SETFL = 4;

const F_GETLK = 5;

const F_SETLK = 6;

const F_SETLKW = 7;

const F_SETOWN = 8;

const F_GETOWN = 9;

const F_UNLCK = 2;

const F_RDLCK = 0;

const F_WRLCK = 1;
