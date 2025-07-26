<?php

namespace Grpc;

use InvalidArgumentException;

const CALL_OK = 0;

const CALL_ERROR = 1;

const CALL_ERROR_NOT_ON_SERVER = 2;

const CALL_ERROR_NOT_ON_CLIENT = 3;

const CALL_ERROR_ALREADY_ACCEPTED = 4;

const CALL_ERROR_ALREADY_INVOKED = 5;

const CALL_ERROR_NOT_INVOKED = 6;

const CALL_ERROR_ALREADY_FINISHED = 7;

const CALL_ERROR_TOO_MANY_OPERATIONS = 8;

const CALL_ERROR_INVALID_FLAGS = 9;

const CALL_ERROR_INVALID_METADATA = 10;

const CALL_ERROR_INVALID_MESSAGE = 11;

const CALL_ERROR_NOT_SERVER_COMPLETION_QUEUE = 12;

const CALL_ERROR_BATCH_TOO_BIG = 13;

const CALL_ERROR_PAYLOAD_TYPE_MISMATCH = 14;

const WRITE_BUFFER_HINT = 1;

const WRITE_NO_COMPRESS = 2;

const STATUS_OK = 0;

const STATUS_CANCELLED = 1;

const STATUS_UNKNOWN = 2;

const STATUS_INVALID_ARGUMENT = 3;

const STATUS_DEADLINE_EXCEEDED = 4;

const STATUS_NOT_FOUND = 5;

const STATUS_ALREADY_EXISTS = 6;

const STATUS_PERMISSION_DENIED = 7;

const STATUS_UNAUTHENTICATED = 16;

const STATUS_RESOURCE_EXHAUSTED = 8;

const STATUS_FAILED_PRECONDITION = 9;

const STATUS_ABORTED = 10;

const STATUS_OUT_OF_RANGE = 11;

const STATUS_UNIMPLEMENTED = 12;

const STATUS_INTERNAL = 13;

const STATUS_UNAVAILABLE = 14;

const STATUS_DATA_LOSS = 15;

const OP_SEND_INITIAL_METADATA = 0;

const OP_SEND_MESSAGE = 1;

const OP_SEND_CLOSE_FROM_CLIENT = 2;

const OP_SEND_STATUS_FROM_SERVER = 3;

const OP_RECV_INITIAL_METADATA = 4;

const OP_RECV_MESSAGE = 5;

const OP_RECV_STATUS_ON_CLIENT = 6;

const OP_RECV_CLOSE_ON_SERVER = 7;

const CHANNEL_IDLE = 0;

const CHANNEL_CONNECTING = 1;

const CHANNEL_READY = 2;

const CHANNEL_TRANSIENT_FAILURE = 3;

const CHANNEL_SHUTDOWN = 4;

const CHANNEL_FATAL_FAILURE = 4;

class Server
{
    public function __construct(array $args) {}

    /**
     * @param int $tag_new
     * @param int $tag_cancel
     */
    public function requestCall($tag_new, $tag_cancel)
    {
    }

    /**
     * @param string $addr
     *
     * @return bool
     */
    public function addHttp2Port($addr)
    {
    }

    /**
     * @param string             $addr
     * @param ServerCredentials $creds_obj
     *
     * @return bool
     */
    public function addSecureHttp2Port($addr, $creds_obj)
    {
    }

    public function start()
    {
    }
}

class ServerCredentials
{
    /**
     * @param string $pem_root_certs
     * @param string $pem_private_key
     * @param string $pem_cert_chain
     *
     * @return object
     * @throws InvalidArgumentException
     */
    public static function createSsl($pem_root_certs, $pem_private_key, $pem_cert_chain)
    {
    }
}

class Channel
{
    /**
     * @param string $target
     * @param array  $args
     *
     * @throws InvalidArgumentException
     */
    public function __construct($target, $args = []) {}

    /**
     * @return string
     */
    public function getTarget()
    {
    }

    /**
     * @param bool $try_to_connect
     *
     * @return int
     * @throws InvalidArgumentException
     */
    public function getConnectivityState($try_to_connect = false)
    {
    }

    /**
     * @param int     $last_state
     * @param Timeval $deadline_obj
     *
     * @return bool
     * @throws InvalidArgumentException
     */
    public function watchConnectivityState($last_state, Timeval $deadline_obj)
    {
    }

    public function close()
    {
    }
}

class ChannelCredentials
{
    /**
     * @param string $pem_roots
     *
     * @throws InvalidArgumentException
     */
    public static function setDefaultRootsPem($pem_roots)
    {
    }

    /**
     * @return ChannelCredentials
     */
    public static function createDefault()
    {
    }

    /**
     * @param string|null $pem_root_certs
     * @param string|null $pem_private_key
     * @param string|null $pem_cert_chain
     *
     * @return ChannelCredentials
     * @throws InvalidArgumentException
     */
    public static function createSsl(
        string $pem_root_certs = null,
        string $pem_private_key = null,
        string $pem_cert_chain = null,
    ) {
    }

    /**
     * @param ChannelCredentials $cred1
     * @param CallCredentials    $cred2
     *
     * @return ChannelCredentials
     * @throws InvalidArgumentException
     */
    public static function createComposite(ChannelCredentials $cred1, CallCredentials $cred2)
    {
    }

    /**
     * @return null
     */
    public static function createInsecure()
    {
    }
}

class Call
{
    /**
     * @param string  $method
     * @param null|string $host_override
     *
     * @throws InvalidArgumentException
     */
    public function __construct(Channel $channel, $method, Timeval $absolute_deadline, $host_override = null) {}

    /**
     * @param array $batch
     *
     * @return object
     * @throws InvalidArgumentException
     * @throws \LogicException
     */
    public function startBatch(array $batch)
    {
    }

    /**
     * @return int
     * @throws InvalidArgumentException
     */
    public function setCredentials(CallCredentials $creds_obj)
    {
    }

    /**
     * @return string
     */
    public function getPeer()
    {
    }

    public function cancel()
    {
    }
}

class CallCredentials
{
    /**
     * @return CallCredentials
     * @throws InvalidArgumentException
     */
    public static function createComposite(CallCredentials $cred1, CallCredentials $cred2)
    {
    }

    /**
     * @return CallCredentials
     * @throws InvalidArgumentException
     */
    public static function createFromPlugin(\Closure $callback)
    {
    }
}

class Timeval
{
    /**
     * @param int $usec
     */
    public function __construct($usec) {}

    /**
     * @return Timeval
     * @throws InvalidArgumentException
     */
    public function add(Timeval $other)
    {
    }

    /**
     * @return int
     * @throws InvalidArgumentException
     */
    public static function compare(Timeval $a, Timeval $b)
    {
    }

    /**
     * @return Timeval
     */
    public static function infFuture()
    {
    }

    /**
     * @return Timeval
     */
    public static function infPast()
    {
    }

    /**
     * @return Timeval
     */
    public static function now()
    {
    }

    /**
     * @return bool
     * @throws InvalidArgumentException
     */
    public static function similar(Timeval $a, Timeval $b, Timeval $threshold)
    {
    }

    public function sleepUntil()
    {
    }

    /**
     * @return Timeval
     * @throws InvalidArgumentException
     */
    public function subtract(Timeval $other)
    {
    }

    /**
     * @return Timeval
     */
    public static function zero()
    {
    }
}
