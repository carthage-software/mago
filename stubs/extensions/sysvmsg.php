<?php

function msg_get_queue(int $key, int $permissions = 0666): SysvMessageQueue|false
{
}

function msg_send(
    SysvMessageQueue $queue,
    int $message_type,
    string|int|float|bool $message,
    bool $serialize = true,
    bool $blocking = true,
    int &$error_code = null,
): bool {
}

function msg_receive(
    SysvMessageQueue $queue,
    int $desired_message_type,
    int &$received_message_type,
    int $max_message_size,
    mixed &$message,
    bool $unserialize = true,
    int $flags = 0,
    int &$error_code = null,
): bool {
}

function msg_remove_queue(SysvMessageQueue $queue): bool
{
}

/**
 * @return false|array{
 *  'msg_perm.uid': int,
 *  'msg_perm.gid': int,
 *  'msg_perm.mode': int,
 *  'msg_stime': int,
 *  'msg_rtime': int,
 *  'msg_ctime': int,
 *  'msg_qnum': int,
 *  'msg_qbytes': int,
 *  'msg_lspid': int,
 *  'msg_lrpid': int
 * }
 */
function msg_stat_queue(SysvMessageQueue $queue): array|false
{
}

function msg_set_queue(SysvMessageQueue $queue, array $data): bool
{
}

function msg_queue_exists(int $key): bool
{
}

const MSG_IPC_NOWAIT = 1;

const MSG_EAGAIN = 11;

const MSG_ENOMSG = 42;

const MSG_NOERROR = 2;

const MSG_EXCEPT = 4;

final class SysvMessageQueue
{
    private function __construct() {}
}
