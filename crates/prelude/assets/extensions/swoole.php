<?php

// Stubs for the Swoole extension (https://swoole.com / https://github.com/swoole/swoole-src).
//
// Consolidated from JetBrains/phpstorm-stubs, which is licensed under Apache-2.0:
//   https://github.com/JetBrains/phpstorm-stubs/tree/master/swoole
// Mago is dual-licensed MIT OR Apache-2.0, so the Apache-2.0 terms apply to this file.

namespace Swoole {
    class Atomic
    {
        public function __construct(int $value = 0) {}

        /** @return int */
        public function add(int $add_value = 1) {}

        /** @return int */
        public function sub(int $sub_value = 1) {}

        /** @return int */
        public function get() {}

        public function set(int $value) {}

        /** @return bool */
        public function wait(float $timeout = 1.0) {}

        /** @return bool */
        public function wakeup(int $count = 1) {}

        /** @return bool */
        public function cmpset(int $cmp_value, int $new_value) {}
    }
}

namespace Swoole\Atomic {
    class Long
    {
        public function __construct(int $value = 0) {}

        /** @return int */
        public function add(int $add_value = 1) {}

        /** @return int */
        public function sub(int $sub_value = 1) {}

        /** @return int */
        public function get() {}

        public function set(int $value) {}

        /** @return bool */
        public function cmpset(int $cmp_value, int $new_value) {}
    }
}

namespace Swoole {
    class Client
    {
        public const MSG_OOB = 1;
        public const MSG_PEEK = 2;
        public const MSG_DONTWAIT = 64;
        public const MSG_WAITALL = 256;
        public const SHUT_RDWR = 2;
        public const SHUT_RD = 0;
        public const SHUT_WR = 1;

        public $errCode = 0;
        public $sock = -1;
        public $reuse = false;
        public $reuseCount = 0;
        public $type = 0;
        public $id;
        public $setting;

        public function __construct($type, $async = null, $id = null) {}

        public function __destruct() {}

        public function set(array $settings) {}

        public function connect($host, $port = null, $timeout = null, $sock_flag = null) {}

        public function recv($size = null, $flag = null) {}

        public function send($data, $flag = null) {}

        public function sendfile($filename, $offset = null, $length = null) {}

        public function sendto($ip, $port, $data) {}

        public function shutdown($how) {}

        public function enableSSL() {}

        public function getPeerCert() {}

        public function verifyPeerCert() {}

        public function isConnected() {}

        public function getsockname() {}

        public function getpeername() {}

        public function close($force = null) {}

        public function getSocket() {}
    }
}

namespace Swoole\Client {
    class Exception extends \Swoole\Exception {}
}

namespace Swoole\Connection {
    class Iterator implements \Iterator, \ArrayAccess, \Countable
    {
        public function __construct() {}

        public function __destruct() {}

        public function rewind(): void {}

        public function next(): void {}

        public function current() {}

        public function key() {}

        public function valid(): bool {}

        public function count(): int {}

        public function offsetExists($fd): bool {}

        public function offsetGet($fd) {}

        public function offsetSet($fd, $value): void {}

        public function offsetUnset($fd): void {}
    }
}

namespace Swoole {
    class Coroutine
    {
        public static function create(callable $func, ...$params) {}

        public static function defer($callback) {}

        /**
         * To set runtime configurations of coroutines.
         *
         * @return void
         */
        public static function set(array $options) {}

        /**
         * To get runtime configurations of coroutines.
         *
         * @return array|null
         */
        public static function getOptions() {}

        public static function exists($cid) {}

        public static function yield() {}

        public static function cancel($cid) {}

        /**
         * Waits for a list of coroutines to finish.
         *
         * This method is similar to class \Swoole\Coroutine\WaitGroup and \Swoole\Coroutine\Barrier. They are different
         * implementations of the same functionality.
         *
         * @param array $cid_array An array of coroutines.
         * @param int $timeout
         * @return bool TRUE if succeeds; otherwise FALSE.
         * @see \Swoole\Coroutine\WaitGroup
         * @see \Swoole\Coroutine\Barrier
         * @since 4.8.0
         */
        public static function join($cid_array, $timeout = -1) {}

        public static function isCanceled() {}

        public static function suspend() {}

        public static function resume($cid) {}

        public static function stats() {}

        public static function getCid() {}

        public static function getuid() {}

        public static function getPcid($cid = null) {}

        public static function getContext($cid = null) {}

        public static function getBackTrace($cid = null, $options = null, $limit = null) {}

        public static function printBackTrace($cid = null, $options = null, $limit = null) {}

        public static function getElapsed($cid = null) {}

        /**
         * Get memory usage of a coroutine.
         *
         * @return int|false Memory usage of the coroutine; FALSE if the specified coroutine doesn't exist.
         * @since 4.8.0
         */
        public static function getStackUsage(?int $cid = null) {}

        public static function list() {}

        public static function listCoroutines() {}

        public static function enableScheduler() {}

        public static function disableScheduler() {}

        public static function gethostbyname($domain_name, $family = null, $timeout = null) {}

        public static function dnsLookup($domain_name, $timeout = null, $type = null) {}

        public static function exec($command, $get_error_stream = null) {}

        public static function sleep($seconds) {}

        public static function getaddrinfo(
            $hostname,
            $family = null,
            $socktype = null,
            $protocol = null,
            $service = null,
            $timeout = null,
        ) {}

        public static function statvfs($path) {}

        public static function readFile($filename) {}

        public static function writeFile($filename, $data, $flags = null) {}

        public static function wait($timeout = null) {}

        public static function waitPid($pid, $timeout = null) {}

        public static function waitSignal($signo, $timeout = null) {}

        public static function waitEvent($fd, $events = null, $timeout = null) {}

        public static function fread($handle, $length = null) {}

        public static function fgets($handle) {}

        public static function fwrite($handle, $string, $length = null) {}
    }
}

namespace Swoole\Coroutine {
    class Channel
    {
        public $capacity = 0;
        public $errCode = 0;

        public function __construct($size = null) {}

        public function push($data, $timeout = null) {}

        public function pop($timeout = null) {}

        public function isEmpty() {}

        public function isFull() {}

        public function close() {}

        public function stats() {}

        public function length() {}
    }
}

namespace Swoole\Coroutine {
    class Client
    {
        public const MSG_OOB = 1;
        public const MSG_PEEK = 2;
        public const MSG_DONTWAIT = 64;
        public const MSG_WAITALL = 256;

        public $errCode = 0;
        public $errMsg = '';
        public $fd = -1;
        public $type = 1;
        public $setting;
        public $connected = false;
        private $socket;

        public function __construct($type) {}

        public function __destruct() {}

        public function set(array $settings) {}

        public function connect($host, $port = null, $timeout = null, $sock_flag = null) {}

        public function recv($timeout = null) {}

        public function peek($length = null) {}

        public function send($data) {}

        public function sendfile($filename, $offset = null, $length = null) {}

        public function sendto($address, $port, $data) {}

        public function recvfrom($length, &$address, &$port = null) {}

        public function enableSSL() {}

        public function getPeerCert() {}

        public function verifyPeerCert() {}

        public function isConnected() {}

        public function getsockname() {}

        public function getpeername() {}

        public function close() {}

        public function exportSocket() {}
    }
}

namespace Swoole\Coroutine {
    class Context extends \ArrayObject
    {
        public const STD_PROP_LIST = 1;
        public const ARRAY_AS_PROPS = 2;
    }
}

namespace Swoole\Coroutine\Curl {
    class Exception extends \Swoole\Exception {}
}

namespace Swoole\Coroutine\Http {
    class Client
    {
        public $errCode = 0;
        public $errMsg = '';
        public $connected = false;
        public $host = '';
        public $port = 0;
        public $ssl = false;
        public $setting;
        public $requestMethod;
        public $requestHeaders;
        public $requestBody;
        public $uploadFiles;
        public $downloadFile;
        public $downloadOffset = 0;
        public $statusCode = 0;
        public $headers;
        public $set_cookie_headers;
        public $cookies;
        public $body = '';

        public function __construct($host, $port = null, $ssl = null) {}

        public function __destruct() {}

        public function set(array $settings) {}

        public function getDefer() {}

        public function setDefer($defer = null) {}

        public function setMethod($method) {}

        public function setHeaders(array $headers) {}

        public function setBasicAuth($username, $password) {}

        public function setCookies(array $cookies) {}

        public function setData($data) {}

        public function addFile($path, $name, $type = null, $filename = null, $offset = null, $length = null) {}

        public function addData($path, $name, $type = null, $filename = null) {}

        public function execute($path) {}

        public function getpeername() {}

        public function getsockname() {}

        public function get($path) {}

        public function post($path, $data) {}

        public function download($path, $file, $offset = null) {}

        public function getBody() {}

        public function getHeaders() {}

        public function getCookies() {}

        public function getStatusCode() {}

        public function getHeaderOut() {}

        public function getPeerCert() {}

        public function upgrade($path) {}

        public function push($data, $opcode = null, $flags = null) {}

        public function recv($timeout = null) {}

        public function close() {}
    }
}

namespace Swoole\Coroutine\Http\Client {
    class Exception extends \Swoole\Exception {}
}

namespace Swoole\Coroutine\Http {
    class Server
    {
        public $fd = -1;
        public $host;
        public $port = -1;
        public $ssl = false;
        public $settings;
        public $errCode = 0;
        public $errMsg = '';

        public function __construct($host, $port = null, $ssl = null, $reuse_port = null) {}

        public function __destruct() {}

        public function set(array $settings) {}

        public function handle($pattern, callable $callback) {}

        public function start() {}

        public function shutdown() {}

        private function onAccept() {}
    }
}

namespace Swoole\Coroutine\Http2 {
    class Client
    {
        public $errCode = 0;
        public $errMsg = 0;
        public $sock = -1;
        public $type = 0;
        public $setting;
        public $connected = false;
        public $host;
        public $port = 0;
        public $ssl = false;

        public function __construct($host, $port = null, $open_ssl = null) {}

        public function __destruct() {}

        public function set(array $settings) {}

        public function connect() {}

        public function stats($key = null) {}

        public function isStreamExist($stream_id) {}

        public function send($request) {}

        public function write($stream_id, $data, $end_stream = null) {}

        public function recv($timeout = null) {}

        public function read($timeout = null) {}

        public function goaway($error_code = null, $debug_data = null) {}

        public function ping() {}

        public function close() {}
    }
}

namespace Swoole\Coroutine\Http2\Client {
    class Exception extends \Swoole\Exception {}
}

namespace Swoole\Coroutine {
    class Iterator extends \ArrayIterator
    {
        public const STD_PROP_LIST = 1;
        public const ARRAY_AS_PROPS = 2;
    }
}

namespace Swoole\Coroutine {
    class MySQL
    {
        public $serverInfo;
        public $sock = -1;
        public $connected = false;
        public $connect_errno = 0;
        public $connect_error = '';
        public $affected_rows = 0;
        public $insert_id = 0;
        public $error = '';
        public $errno = 0;

        public function __construct() {}

        public function __destruct() {}

        public function getDefer() {}

        public function setDefer($defer = null) {}

        public function connect(?array $server_config = null) {}

        public function query($sql, $timeout = null) {}

        public function fetch() {}

        public function fetchAll() {}

        public function nextResult() {}

        public function prepare($query, $timeout = null) {}

        public function recv() {}

        public function begin($timeout = null) {}

        public function commit($timeout = null) {}

        public function rollback($timeout = null) {}

        public function escape($string, $flags = null) {}

        public function close() {}
    }
}

namespace Swoole\Coroutine\MySQL {
    class Exception extends \Swoole\Exception {}
}

namespace Swoole\Coroutine\MySQL {
    class Statement
    {
        public $id = 0;
        public $affected_rows = 0;
        public $insert_id = 0;
        public $error = '';
        public $errno = 0;

        public function execute($params = null, $timeout = null) {}

        public function fetch($timeout = null) {}

        public function fetchAll($timeout = null) {}

        public function nextResult($timeout = null) {}

        public function recv($timeout = null) {}

        public function close() {}
    }
}

namespace Swoole\Coroutine {
    class Redis
    {
        public $host = '';
        public $port = 0;
        public $setting;
        public $sock = -1;
        public $connected = false;
        public $errType = 0;
        public $errCode = 0;
        public $errMsg = '';

        public function __construct($config = null) {}

        public function __destruct() {}

        public function connect($host, $port = null, $serialize = null) {}

        public function getAuth() {}

        public function getDBNum() {}

        public function getOptions() {}

        public function setOptions($options) {}

        public function getDefer() {}

        public function setDefer($defer) {}

        public function recv() {}

        public function request(array $params) {}

        public function close() {}

        public function set($key, $value, $timeout = null, $opt = null) {}

        public function setBit($key, $offset, $value) {}

        public function setEx($key, $expire, $value) {}

        public function psetEx($key, $expire, $value) {}

        public function lSet($key, $index, $value) {}

        public function get($key) {}

        public function mGet($keys) {}

        public function del($key, $other_keys = null) {}

        public function hDel($key, $member, $other_members = null) {}

        public function hSet($key, $member, $value) {}

        public function hMSet($key, $pairs) {}

        public function hSetNx($key, $member, $value) {}

        public function delete($key, $other_keys = null) {}

        public function mSet($pairs) {}

        public function mSetNx($pairs) {}

        public function getKeys($pattern) {}

        public function keys($pattern) {}

        public function exists($key, $other_keys = null) {}

        public function type($key) {}

        public function strLen($key) {}

        public function lPop($key) {}

        public function blPop($key, $timeout_or_key, $extra_args = null) {}

        public function rPop($key) {}

        public function brPop($key, $timeout_or_key, $extra_args = null) {}

        public function bRPopLPush($src, $dst, $timeout) {}

        public function lSize($key) {}

        public function lLen($key) {}

        public function sSize($key) {}

        public function scard($key) {}

        public function sPop($key) {}

        public function sMembers($key) {}

        public function sGetMembers($key) {}

        public function sRandMember($key, $count = null) {}

        public function persist($key) {}

        public function ttl($key) {}

        public function pttl($key) {}

        public function zCard($key) {}

        public function zSize($key) {}

        public function hLen($key) {}

        public function hKeys($key) {}

        public function hVals($key) {}

        public function hGetAll($key) {}

        public function debug($key) {}

        public function restore($ttl, $key, $value) {}

        public function dump($key) {}

        public function renameKey($key, $newkey) {}

        public function rename($key, $newkey) {}

        public function renameNx($key, $newkey) {}

        public function rpoplpush($src, $dst) {}

        public function randomKey() {}

        public function pfadd($key, $elements) {}

        public function pfcount($key) {}

        public function pfmerge($dstkey, $keys) {}

        public function ping() {}

        public function auth($password) {}

        public function unwatch() {}

        public function watch($key, $other_keys = null) {}

        public function save() {}

        public function bgSave() {}

        public function lastSave() {}

        public function flushDB() {}

        public function flushAll() {}

        public function dbSize() {}

        public function bgrewriteaof() {}

        public function time() {}

        public function role() {}

        public function setRange($key, $offset, $value) {}

        public function setNx($key, $value) {}

        public function getSet($key, $value) {}

        public function append($key, $value) {}

        public function lPushx($key, $value) {}

        public function lPush($key, $value) {}

        public function rPush($key, $value) {}

        public function rPushx($key, $value) {}

        public function sContains($key, $value) {}

        public function sismember($key, $value) {}

        public function zScore($key, $member) {}

        public function zRank($key, $member) {}

        public function zRevRank($key, $member) {}

        public function hGet($key, $member) {}

        public function hMGet($key, $keys) {}

        public function hExists($key, $member) {}

        public function publish($channel, $message) {}

        public function zIncrBy($key, $value, $member) {}

        public function zAdd($key, $score, $value) {}

        public function zPopMin($key, $count = null) {}

        public function zPopMax($key, $count = null) {}

        public function bzPopMin($key, $timeout_or_key, $extra_args = null) {}

        public function bzPopMax($key, $timeout_or_key, $extra_args = null) {}

        public function zDeleteRangeByScore($key, $min, $max) {}

        public function zRemRangeByScore($key, $min, $max) {}

        public function zCount($key, $min, $max) {}

        public function zRange($key, $start, $end, $scores = null) {}

        public function zRevRange($key, $start, $end, $scores = null) {}

        public function zRangeByScore($key, $start, $end, $options = null) {}

        public function zRevRangeByScore($key, $start, $end, $options = null) {}

        public function zRangeByLex($key, $min, $max, $offset = null, $limit = null) {}

        public function zRevRangeByLex($key, $min, $max, $offset = null, $limit = null) {}

        public function zInter($key, $keys, $weights = null, $aggregate = null) {}

        public function zinterstore($key, $keys, $weights = null, $aggregate = null) {}

        public function zUnion($key, $keys, $weights = null, $aggregate = null) {}

        public function zunionstore($key, $keys, $weights = null, $aggregate = null) {}

        public function incrBy($key, $value) {}

        public function hIncrBy($key, $member, $value) {}

        public function incr($key) {}

        public function decrBy($key, $value) {}

        public function decr($key) {}

        public function getBit($key, $offset) {}

        public function lInsert($key, $position, $pivot, $value) {}

        public function lGet($key, $index) {}

        public function lIndex($key, $integer) {}

        public function setTimeout($key, $timeout) {}

        public function expire($key, $integer) {}

        public function pexpire($key, $timestamp) {}

        public function expireAt($key, $timestamp) {}

        public function pexpireAt($key, $timestamp) {}

        public function move($key, $dbindex) {}

        public function select($dbindex) {}

        public function getRange($key, $start, $end) {}

        public function listTrim($key, $start, $stop) {}

        public function ltrim($key, $start, $stop) {}

        public function lGetRange($key, $start, $end) {}

        public function lRange($key, $start, $end) {}

        public function lRem($key, $value, $count = 0) {}

        public function lRemove($key, $value, $count) {}

        public function zDeleteRangeByRank($key, $start, $end) {}

        public function zRemRangeByRank($key, $min, $max) {}

        public function incrByFloat($key, $value) {}

        public function hIncrByFloat($key, $member, $value) {}

        public function bitCount($key) {}

        public function bitOp($operation, $ret_key, $key, $other_keys = null) {}

        public function sAdd($key, $value) {}

        public function sMove($src, $dst, $value) {}

        public function sDiff($key, $other_keys = null) {}

        public function sDiffStore($dst, $key, $other_keys = null) {}

        public function sUnion($key, $other_keys = null) {}

        public function sUnionStore($dst, $key, $other_keys = null) {}

        public function sInter($key, $other_keys = null) {}

        public function sInterStore($dst, $key, $other_keys = null) {}

        public function sRemove($key, $value) {}

        public function srem($key, $value) {}

        public function zDelete($key, $member, $other_members = null) {}

        public function zRemove($key, $member, $other_members = null) {}

        public function zRem($key, $member, $other_members = null) {}

        public function pSubscribe($patterns) {}

        public function subscribe($channels) {}

        public function unsubscribe($channels) {}

        public function pUnSubscribe($patterns) {}

        public function multi() {}

        public function exec() {}

        public function eval($script, $args = null, $num_keys = null) {}

        public function evalSha($script_sha, $args = null, $num_keys = null) {}

        public function script($cmd, $args = null) {}

        /**
         * @return int
         * @see https://redis.io/commands/xlen
         * @since 4.8.0
         */
        public function xLen(string $key) {}

        /**
         * @return void|false Returns FALSE if parameter $pairs is empty; otherwise nothing returns.
         * @see https://redis.io/commands/xadd
         * @since 4.8.0
         */
        public function xAdd(string $key, string $id, array $pairs, array $options = []) {}

        /**
         * @return array|false Returns FALSE if error happens or parameter $streams is empty; otherwise, an array is returned.
         * @see https://redis.io/commands/xread
         * @since 4.8.0
         */
        public function xRead(array $streams, array $options = []) {}

        /**
         * @return int The number of entries actually deleted.
         * @see https://redis.io/commands/xdel
         * @since 4.8.0
         */
        public function xDel(string $key, string $id) {}

        /**
         * @return array
         * @see https://redis.io/commands/xrange
         * @since 4.8.0
         */
        public function xRange(string $key, string $start, string $end, int $count = 0) {}

        /**
         * @return array
         * @see https://redis.io/commands/xrevrange
         * @since 4.8.0
         */
        public function xRevRange(string $key, string $start, string $end, int $count = 0) {}

        /**
         * @return array|false Returns FALSE if error happens; otherwise, an array is returned.
         * @see https://redis.io/commands/xtrim
         * @since 4.8.0
         */
        public function xTrim(string $key, array $options = []) {}

        /**
         * @return string
         * @see https://redis.io/commands/xgroup
         * @since 4.8.0
         */
        public function xGroupCreate(string $key, string $group_name, string $id, bool $mkstream = false) {}

        /**
         * @return string
         * @see https://redis.io/commands/xgroup
         * @since 4.8.0
         */
        public function xGroupSetId(string $key, string $group_name, string $id) {}

        /**
         * @return int The number of destroyed consumer groups (0 or 1).
         * @see https://redis.io/commands/xgroup
         * @since 4.8.0
         */
        public function xGroupDestroy(string $key, string $group_name) {}

        /**
         * @return int The number of created consumers (0 or 1).
         * @see https://redis.io/commands/xgroup
         * @since 4.8.0
         */
        public function xGroupCreateConsumer(string $key, string $group_name, string $consumer_name) {}

        /**
         * @return int The number of pending messages that the consumer had before it was deleted.
         * @see https://redis.io/commands/xgroup
         * @since 4.8.0
         */
        public function xGroupDelConsumer(string $key, string $group_name, string $consumer_name) {}

        /**
         * @return array|false Returns FALSE if error happens; otherwise, an array is returned.
         * @see https://redis.io/commands/xreadgroup
         * @since 4.8.0
         */
        public function xReadGroup(string $group_name, string $consumer_name, array $streams, array $options = []) {}

        /**
         * @return array|false Returns FALSE if error happens; otherwise, an array is returned.
         * @see https://redis.io/commands/xpending
         * @since 4.8.0
         */
        public function xPending(string $key, string $group_name, array $options = []) {}

        /**
         * @return array|false Returns FALSE if error happens or parameter $id is empty; otherwise, an array is returned.
         * @see https://redis.io/commands/xack
         * @since 4.8.0
         */
        public function xAck(string $key, string $group_name, array $id) {}

        /**
         * @return array|false Returns FALSE if error happens; otherwise, an array is returned.
         * @see https://redis.io/commands/xclaim
         * @since 4.8.0
         */
        public function xClaim(
            string $key,
            string $group_name,
            string $consumer_name,
            int $min_idle_time,
            array $id,
            array $options = [],
        ) {}

        /**
         * @return array|false Returns FALSE if error happens; otherwise, an array is returned.
         * @see https://redis.io/commands/xautoclaim
         * @since 4.8.0
         */
        public function xAutoClaim(
            string $key,
            string $group_name,
            string $consumer_name,
            int $min_idle_time,
            string $start,
            array $options = [],
        ) {}

        /**
         * @return array
         * @see https://redis.io/commands/xinfo
         * @since 4.8.0
         */
        public function xInfoConsumers(string $key, string $group_name) {}

        /**
         * @return array
         * @see https://redis.io/commands/xinfo
         * @since 4.8.0
         */
        public function xInfoGroups(string $key) {}

        /**
         * @return array
         * @see https://redis.io/commands/xinfo
         * @since 4.8.0
         */
        public function xInfoStream(string $key) {}
    }
}

namespace Swoole\Coroutine {
    class Scheduler
    {
        /**
         * Add a task (implemented in a callback).
         *
         * @return false|void Returns FALSE if the scheduler has already been started; otherwise nothing returns.
         * @see \Swoole\Coroutine\Scheduler::start()
         */
        public function add(callable $func, ...$params) {}

        /**
         * Add a list of tasks (implemented in callbacks).
         *
         * @return false|void Returns FALSE if the scheduler has already been started; otherwise nothing returns.
         * @see \Swoole\Coroutine\Scheduler::start()
         */
        public function parallel(int $n, callable $func, ...$params) {}

        /**
         * To set runtime configurations of coroutines.
         *
         * This method is an alias of method \Swoole\Coroutine::set().
         *
         * @return void
         * @see \Swoole\Coroutine::set()
         */
        public function set(array $settings) {}

        /**
         * To get runtime configurations of coroutines.
         *
         * This method is an alias of method \Swoole\Coroutine::getOptions().
         *
         * @return array|null
         * @see \Swoole\Coroutine::getOptions()
         * @since Swoole 4.6.0
         */
        public function getOptions() {}

        /**
         * Start running the list of tasks (callbacks) added through method add() and parallel().
         *
         * @return bool
         * @see \Swoole\Coroutine\Scheduler::add()
         * @see \Swoole\Coroutine\Scheduler::parallel()
         */
        public function start() {}
    }
}

namespace Swoole\Coroutine {
    use Swoole\Client;

    class Socket
    {
        public $fd = -1;
        public $domain = 0;
        public $type = 0;
        public $protocol = 0;
        public $errCode = 0;
        public $errMsg = '';

        public function __construct($domain, $type, $protocol = null) {}

        public function bind($address, $port = null) {}

        public function listen($backlog = null) {}

        public function accept($timeout = null) {}

        public function connect($host, $port = null, $timeout = null) {}

        public function checkLiveness() {}

        public function peek($length = null) {}

        public function recv($length = null, $timeout = null) {}

        public function recvAll($length = null, $timeout = null) {}

        public function recvLine($length = null, $timeout = null) {}

        public function recvWithBuffer($length = null, $timeout = null) {}

        public function recvPacket($timeout = null) {}

        public function send($data, $timeout = null) {}

        public function readVector($io_vector, $timeout = null) {}

        public function readVectorAll($io_vector, $timeout = null) {}

        public function writeVector($io_vector, $timeout = null) {}

        public function writeVectorAll($io_vector, $timeout = null) {}

        public function sendFile($filename, $offset = null, $length = null) {}

        public function sendAll($data, $timeout = null) {}

        public function recvfrom(&$peername, $timeout = null) {}

        public function sendto($addr, $port, $data) {}

        public function getOption($level, $opt_name) {}

        public function setProtocol(array $settings): bool {}

        public function setOption($level, $opt_name, $opt_value) {}

        public function sslHandshake(): bool {}

        public function shutdown(int $how = Client::SHUT_RDWR): bool {}

        public function cancel(int $event = SWOOLE_EVENT_READ): bool {}

        public function close(): bool {}

        /** @return array|false If succeeds, return an array with two fields in it: "address" and "port"; otherwise, return FALSE. */
        public function getpeername() {}

        /** @return array|false If succeeds, return an array with two fields in it: "address" and "port"; otherwise, return FALSE. */
        public function getsockname() {}

        /** @since 4.8.3 */
        public function isClosed(): bool {}
    }
}

namespace Swoole\Coroutine\Socket {
    class Exception extends \Swoole\Exception {}
}

namespace Swoole\Coroutine {
    class System
    {
        public static function gethostbyname($domain_name, $family = null, $timeout = null) {}

        public static function dnsLookup($domain_name, $timeout = null, $type = null) {}

        public static function exec($command, $get_error_stream = null) {}

        public static function sleep($seconds) {}

        public static function getaddrinfo(
            $hostname,
            $family = null,
            $socktype = null,
            $protocol = null,
            $service = null,
            $timeout = null,
        ) {}

        public static function statvfs($path) {}

        public static function readFile($filename) {}

        public static function writeFile($filename, $data, $flags = null) {}

        public static function wait($timeout = null) {}

        public static function waitPid($pid, $timeout = null) {}

        public static function waitSignal($signo, $timeout = null) {}

        public static function waitEvent($fd, $events = null, $timeout = null) {}

        public static function fread($handle, $length = null) {}

        public static function fwrite($handle, $string, $length = null) {}

        public static function fgets($handle) {}
    }
}

namespace Swoole {
    class Error extends \Error {}
}

namespace Swoole {
    class Event
    {
        public static function add($fd, ?callable $read_callback, ?callable $write_callback = null, $events = null) {}

        public static function del($fd) {}

        public static function set(
            $fd,
            ?callable $read_callback = null,
            ?callable $write_callback = null,
            $events = null,
        ) {}

        public static function isset($fd, $events = null) {}

        public static function dispatch() {}

        /** @return true */
        public static function defer(callable $callback) {}

        public static function cycle(?callable $callback, $before = null) {}

        public static function write($fd, $data) {}

        public static function wait() {}

        public static function rshutdown() {}

        public static function exit() {}
    }
}

namespace Swoole {
    class Exception extends \Exception {}
}

namespace Swoole {
    class ExitException extends Exception
    {
        private $flags = 0;
        private $status = 0;

        public function getFlags() {}

        public function getStatus() {}
    }
}

namespace Swoole\Http {
    class Request
    {
        public $fd = 0;
        public $streamId = 0;
        public $header;
        public $server;
        public $cookie;
        public $get;
        public $files;
        public $post;
        public $tmpfiles;

        public function __destruct() {}

        /**
         * Get the request content, kind of like function call fopen('php://input').
         *
         * This method has an alias of \Swoole\Http\Request::rawContent().
         *
         * @return string|false Return the request content back; return FALSE when error happens.
         * @see \Swoole\Http\Request::rawContent()
         * @since 4.5.0
         */
        public function getContent() {}

        /**
         * Get the request content, kind of like function call fopen('php://input').
         *
         * Alias of method \Swoole\Http\Request::getContent().
         *
         * @return string|false Return the request content back; return FALSE when error happens.
         * @see \Swoole\Http\Request::getContent()
         */
        public function rawContent() {}

        public function getData() {}

        public static function create($options = null) {}

        public function parse($data) {}

        public function isCompleted() {}

        public function getMethod() {}
    }
}

namespace Swoole\Http {
    class Response
    {
        public $fd = 0;
        public $socket;
        public $header;
        public $cookie;
        public $trailer;

        public function __destruct() {}

        public function initHeader() {}

        public function isWritable() {}

        public function cookie(
            $name,
            $value = null,
            $expires = null,
            $path = null,
            $domain = null,
            $secure = null,
            $httponly = null,
            $samesite = null,
            $priority = null,
        ) {}

        public function setCookie(
            $name,
            $value = null,
            $expires = null,
            $path = null,
            $domain = null,
            $secure = null,
            $httponly = null,
            $samesite = null,
            $priority = null,
        ) {}

        public function rawcookie(
            $name,
            $value = null,
            $expires = null,
            $path = null,
            $domain = null,
            $secure = null,
            $httponly = null,
            $samesite = null,
            $priority = null,
        ) {}

        public function status($http_code, $reason = null) {}

        public function setStatusCode($http_code, $reason = null) {}

        public function header($key, $value, $format = null) {}

        public function setHeader($key, $value, $format = null) {}

        public function trailer($key, $value) {}

        public function ping() {}

        public function goaway() {}

        public function write($content) {}

        public function end($content = null) {}

        public function sendfile($filename, $offset = null, $length = null) {}

        public function redirect($location, $http_code = null) {}

        public function detach() {}

        public static function create($server, $fd = null) {}

        public function upgrade() {}

        public function push($data, $opcode = null, $flags = null) {}

        public function recv() {}

        public function close() {}
    }
}

namespace Swoole\Http {
    class Server extends \Swoole\Server {}
}

namespace Swoole\Http2 {
    class Request
    {
        public $path = '/';
        public $method = 'GET';
        public $headers;
        public $cookies;
        public $data = '';
        public $pipeline = false;
    }
}

namespace Swoole\Http2 {
    class Response
    {
        public $streamId = 0;
        public $errCode = 0;
        public $statusCode = 0;
        public $pipeline = false;
        public $headers;
        public $set_cookie_headers;
        public $cookies;
        public $data;
    }
}

namespace Swoole {
    class Lock
    {
        public const FILELOCK = 2;
        public const MUTEX = 3;
        public const SEM = 4;
        public const RWLOCK = 1;
        public const SPINLOCK = 5;

        public $errCode = 0;

        public function __construct(int $type = self::MUTEX, string $filename = '') {}

        /** @return bool */
        public function lock() {}

        /** @return bool */
        public function lockwait(float $timeout = 1.0) {}

        /** @return bool */
        public function trylock() {}

        /** @return bool */
        public function lock_read() {}

        /** @return bool */
        public function trylock_read() {}

        /** @return bool */
        public function unlock() {}

        public function destroy() {}
    }
}

namespace Swoole {
    class Process
    {
        public const IPC_NOWAIT = 256;
        public const PIPE_MASTER = 1;
        public const PIPE_WORKER = 2;
        public const PIPE_READ = 3;
        public const PIPE_WRITE = 4;

        public $pipe;
        public $msgQueueId;
        public $msgQueueKey;

        /**
         * Process ID. This is to uniquely identify the process in the OS.
         *
         * @var int
         */
        public $pid;

        /**
         * ID of the process.
         *
         * In a Swoole program (e.g., a Swoole-based server), there are different types of processes, including event worker
         * processes, task worker processes, and user worker processes. This ID is to uniquely identify the process in the
         * running Swoole program.
         *
         * @var int
         */
        public $id;
        private $callback;

        public function __construct(
            callable $callback,
            $redirect_stdin_and_stdout = null,
            $pipe_type = null,
            $enable_coroutine = null,
        ) {}

        public function __destruct() {}

        public static function wait($blocking = null) {}

        public static function signal($signal_no, $callback) {}

        public static function alarm($usec, $type = null) {}

        public static function kill($pid, $signal_no = null) {}

        public static function daemon($nochdir = null, $noclose = null, $pipes = null) {}

        public function setPriority($which, $priority) {}

        public function getPriority($which) {}

        public function set(array $settings) {}

        public function setTimeout($seconds) {}

        public function setBlocking($blocking) {}

        public function useQueue($key = null, $mode = null, $capacity = null) {}

        public function statQueue() {}

        public function freeQueue() {}

        public function start() {}

        public function write($data) {}

        public function close() {}

        public function read($size = null) {}

        public function push($data) {}

        public function pop($size = null) {}

        public function exit($exit_code = null) {}

        public function exec($exec_file, $args) {}

        public function exportSocket() {}

        public function name($process_name) {}
    }
}

namespace Swoole\Process {
    class Pool
    {
        public $master_pid = -1;
        public $workers;

        public function __construct($worker_num, $ipc_type = null, $msgqueue_key = null, $enable_coroutine = null) {}

        public function __destruct() {}

        public function set(array $settings) {}

        public function on($event_name, callable $callback) {}

        public function getProcess($worker_id = null) {}

        public function listen($host, $port = null, $backlog = null) {}

        public function write($data) {}

        public function detach() {}

        public function start() {}

        public function stop() {}

        public function shutdown() {}
    }
}

namespace Swoole\Redis {
    class Server extends \Swoole\Server
    {
        /**
         * To return an ERR reply from the Redis server.
         *
         * @see \Swoole\Redis\Server::format()
         */
        public const ERROR = 0;

        /**
         * To return a NULL reply from the Redis server.
         *
         * When used as the 1st parameter "$type" in method \Swoole\Redis\Server::format(), there is no need to pass in the
         * 2nd parameter "$value".
         *
         * @see \Swoole\Redis\Server::format()
         */
        public const NIL = 1;

        /**
         * To return a Status reply from the Redis server.
         *
         * @see \Swoole\Redis\Server::format()
         */
        public const STATUS = 2;

        /**
         * To return an Integer reply from the Redis server.
         *
         * When used as the 1st parameter "$type" in method \Swoole\Redis\Server::format(), the 2nd parameter "$value" must
         * be an integer.
         *
         * @see \Swoole\Redis\Server::format()
         */
        public const INT = 3;

        /**
         * To return a String reply from the Redis server.
         *
         * When used as the 1st parameter "$type" in method \Swoole\Redis\Server::format(), the 2nd parameter "$value" must
         * be a string.
         *
         * @see \Swoole\Redis\Server::format()
         */
        public const STRING = 4;

        /**
         * To return a Set reply from the Redis server.
         *
         * When used as the 1st parameter "$type" in method \Swoole\Redis\Server::format(), the 2nd parameter "$value" must
         * be an array.
         *
         * @see \Swoole\Redis\Server::format()
         */
        public const SET = 5;

        /**
         * To return a Map reply from the Redis server.
         *
         * When used as the 1st parameter "$type" in method \Swoole\Redis\Server::format(), the 2nd parameter "$value" must
         * be an associative array.
         *
         * @see \Swoole\Redis\Server::format()
         */
        public const MAP = 6;

        /**
         * Set a handler (a callback function) to process a given Redis command.
         *
         * @return bool TRUE on success, or FALSE on failure.
         */
        public function setHandler(string $command, callable $callback) {}

        /**
         * @param string $command
         * @return callable|null Returns the callback function if defined, otherwise NULL.
         */
        public function getHandler($command) {}

        /**
         * Format a reply.
         *
         * @return string|false
         */
        public static function format(int $type, $value = null) {}
    }
}

namespace Swoole {
    class Runtime
    {
        /**
         * To enable/disable runtime hooks in coroutines.
         *
         * For backward-compatible reason, there are four different ways to call this method:
         *   #1. Swoole\Runtime::enableCoroutine();             // Enable runtime hooks represented by constant SWOOLE_HOOK_ALL.
         *   #2. Swoole\Runtime::enableCoroutine($flags);       // Enable specified runtime hooks.
         *   #3. Swoole\Runtime::enableCoroutine(true, $flags); // Enable specified runtime hooks.
         *   #4. Swoole\Runtime::enableCoroutine(false);        // Disable runtime hooks.
         * Following statements are of the same (when used to disable runtime hooks):
         *   Swoole\Runtime::enableCoroutine(0);       // #2
         *   Swoole\Runtime::enableCoroutine(true, 0); // #3
         *   Swoole\Runtime::enableCoroutine(false);   // #4
         *
         * @param int|bool $enable
         * @return bool TRUE on success, or FALSE on failure.
         */
        public static function enableCoroutine($enable = true, int $flags = SWOOLE_HOOK_ALL) {}

        /** @return int */
        public static function getHookFlags() {}

        /** @return bool true on success or false on failure */
        public static function setHookFlags(int $flags) {}
    }
}

namespace Swoole {
    class Server
    {
        public $setting;
        public $connections;
        public $host = '';
        public $port = 0;
        public $type = 0;
        public $mode = 0;
        public $ports;
        public $master_pid = 0;
        public $manager_pid = 0;
        public $worker_id = -1;
        public $taskworker = false;
        public $worker_pid = 0;
        public $stats_timer;

        /**
         * @var \Swoole\Coroutine\Http\Server
         * @since 4.8.0
         */
        public $admin_server;

        /** @var callable */
        private $onStart;

        /**
         * @var callable
         * @since 4.8.0
         */
        private $onBeforeShutdown;

        /** @var callable */
        private $onShutdown;

        /** @var callable */
        private $onWorkerStart;

        /** @var callable */
        private $onWorkerStop;

        /** @var callable */
        private $onBeforeReload;

        /** @var callable */
        private $onAfterReload;

        /** @var callable */
        private $onWorkerExit;

        /** @var callable */
        private $onWorkerError;

        /** @var callable */
        private $onTask;

        /** @var callable */
        private $onFinish;

        /** @var callable */
        private $onManagerStart;

        /** @var callable */
        private $onManagerStop;

        /** @var callable */
        private $onPipeMessage;

        public function __construct($host, $port = null, $mode = null, $sock_type = null) {}

        public function __destruct() {}

        public function listen($host, $port, $sock_type) {}

        public function addlistener($host, $port, $sock_type) {}

        public function on($event_name, callable $callback) {}

        public function getCallback($event_name) {}

        public function set(array $settings) {}

        public function start() {}

        public function send($fd, $send_data, $server_socket = null) {}

        public function sendto($ip, $port, $send_data, $server_socket = null) {}

        public function sendwait($conn_fd, $send_data) {}

        public function exists($fd) {}

        public function exist($fd) {}

        public function protect($fd, $is_protected = null) {}

        public function sendfile($conn_fd, $filename, $offset = null, $length = null) {}

        public function close($fd, $reset = null) {}

        public function confirm($fd) {}

        public function pause($fd) {}

        public function resume($fd) {}

        public function task($data, $worker_id = null, ?callable $finish_callback = null) {}

        public function taskwait($data, $timeout = null, $worker_id = null) {}

        public function taskWaitMulti(array $tasks, $timeout = null) {}

        public function taskCo(array $tasks, $timeout = null) {}

        public function finish($data) {}

        public function reload() {}

        public function shutdown() {}

        public function stop($worker_id = null) {}

        public function getLastError() {}

        public function heartbeat($reactor_id) {}

        public function getClientInfo($fd, $reactor_id = null) {}

        public function getClientList($start_fd, $find_count = null) {}

        /**
         * Get the ID of current worker (either an event worker or a task worker).
         *
         * @return int|false Returns the ID of current worker. Returns false if not called within a worker process (either
         *                   an event worker process or a task worker process).
         */
        public function getWorkerId() {}

        /**
         * Get the process ID of a given worker process (specified by a worker ID).
         *
         * @return int|false Returns the process ID of a given worker process (specified by a worker ID). If the worker ID
         *                   is a negative integer or not passed in, returns the process ID of current worker process.
         *                   Returns false if something wrong happens (e.g., the worker process doesn't exist, or an invalid
         *                   worker ID specified.).
         */
        public function getWorkerPid(int $worker_id = -1) {}

        public function getWorkerStatus($worker_id = null) {}

        /**
         * Run a customized command in a specified process of Swoole.
         *
         * @param bool $json_encode If the callback function of the command returns a JSON encoded string back, it can be decoded automatically by setting this parameter to TRUE.
         * @return mixed|false
         * @see \Swoole\Server::addCommand()
         * @since 4.8.0
         */
        public function command(string $name, int $process_id, int $process_type, $data, bool $json_decode = true) {}

        /**
         * Add a customized command.
         *
         * Commands can be added to the master process, the manager process, or worker processes. Commands can only be added
         * before the server is started.
         *
         * @return bool TRUE if succeeds, otherwise FALSE.
         * @see \Swoole\Server::command()
         * @see SWOOLE_SERVER_COMMAND_MASTER
         * @see SWOOLE_SERVER_COMMAND_MANAGER
         * @see SWOOLE_SERVER_COMMAND_EVENT_WORKER
         * @see SWOOLE_SERVER_COMMAND_TASK_WORKER
         * @since 4.8.0
         */
        public function addCommand(string $name, int $accepted_process_types, callable $callback) {}

        public function getManagerPid() {}

        public function getMasterPid() {}

        public function connection_info($fd, $reactor_id = null) {}

        public function connection_list($start_fd, $find_count = null) {}

        public function sendMessage($message, $dst_worker_id) {}

        /**
         * @return int|false Return the ID of the process (\Swoole\Process::$id) back if succeeds; otherwise return FALSE.
         * @see \Swoole\Process::$id
         */
        public function addProcess(Process $process) {}

        public function stats() {}

        public function getSocket($port = null) {}

        public function bind($fd, $uid) {}

        /**
         * Alias of method \Swoole\Timer::after().
         *
         * @return int
         * @see \Swoole\Timer::after()
         */
        public function after(int $ms, callable $callback, ...$params) {}

        /**
         * Alias of method \Swoole\Timer::tick().
         *
         * @return int
         * @see \Swoole\Timer::tick()
         */
        public function tick(int $ms, callable $callback, ...$params) {}

        /**
         * Alias of method \Swoole\Timer::clear().
         *
         * @return bool
         * @see \Swoole\Timer::clear()
         */
        public function clearTimer(int $timer_id) {}

        /**
         * Alias of method \Swoole\Event::defer().
         *
         * @return true
         * @see \Swoole\Event::defer()
         */
        public function defer(callable $callback) {}
    }
}

namespace Swoole\Server {
    class Event
    {
        public $reactor_id = 0;
        public $fd = 0;
        public $dispatch_time = 0;
        public $data;
    }
}

namespace Swoole\Server {
    class Packet
    {
        public $server_socket = 0;
        public $server_port = 0;
        public $dispatch_time = 0;
        public $address;
        public $port = 0;
    }
}

namespace Swoole\Server {
    class PipeMessage
    {
        public $source_worker_id = 0;
        public $dispatch_time = 0;
        public $data;
    }
}

namespace Swoole\Server {
    class Port
    {
        public $host;
        public $port = 0;
        public $type = 0;
        public $sock = -1;
        public $setting;
        public $connections;
        private $onConnect;
        private $onReceive;
        private $onClose;
        private $onPacket;
        private $onBufferFull;
        private $onBufferEmpty;
        private $onRequest;
        private $onHandShake;
        private $onOpen;
        private $onMessage;
        private $onDisconnect;

        private function __construct() {}

        public function __destruct() {}

        public function set(array $settings) {}

        public function on($event_name, callable $callback) {}

        public function getCallback($event_name) {}

        public function getSocket() {}
    }
}

namespace Swoole\Server {
    class StatusInfo
    {
        public $worker_id = 0;
        public $worker_pid = 0;
        public $status = 0;
        public $exit_code = 0;
        public $signal = 0;
    }
}

namespace Swoole\Server {
    class Task
    {
        public $data;
        public $dispatch_time = 0;
        public $id = -1;
        public $worker_id = -1;
        public $flags = 0;

        public function finish($data) {}

        public static function pack($data) {}
    }
}

namespace Swoole\Server {
    class TaskResult
    {
        public $task_id = 0;
        public $task_worker_id = 0;
        public $dispatch_time = 0;
        public $data;
    }
}

namespace Swoole {
    class Table implements \Iterator, \ArrayAccess, \Countable
    {
        public const TYPE_INT = 1;
        public const TYPE_STRING = 3;
        public const TYPE_FLOAT = 2;

        /** @var int */
        public $size;

        /** @var int */
        public $memorySize;

        public function __construct(int $table_size, float $conflict_proportion = 0.2) {}

        /** @return bool */
        public function column(string $name, int $type, int $size = 0) {}

        /** @return bool */
        public function create() {}

        /** @return bool returns TRUE all the time */
        public function destroy() {}

        /** @return bool */
        public function set(string $key, array $value) {}

        /**
         * @return array|false Return an array of stats information; Return FALSE when error happens.
         * @since 4.8.0
         */
        public function stats() {}

        public function get(string $key, ?string $field = null) {}

        /**
         * This method has an alias of \Swoole\Table::delete().
         *
         * @return bool
         * @see \Swoole\Table::delete()
         */
        public function del(string $key) {}

        /**
         * Alias of method \Swoole\Table::del().
         *
         * @return bool
         * @see \Swoole\Table::del()
         */
        public function delete(string $key) {}

        /**
         * This method has an alias of \Swoole\Table::exist().
         *
         * @return bool
         * @see \Swoole\Table::exist()
         */
        public function exists(string $key) {}

        /**
         * Alias of method \Swoole\Table::exists().
         *
         * @return bool
         * @see \Swoole\Table::exists()
         */
        public function exist(string $key) {}

        /** @return int */
        public function incr(string $key, string $column, $incrby = 1) {}

        /** @return int */
        public function decr(string $key, string $column, $decrby = 1) {}

        /** @return int */
        public function getSize() {}

        /** @return int */
        public function getMemorySize() {}

        /**
         * @see \Iterator::current()
         * @see https://www.php.net/manual/en/iterator.current.php
         * {@inheritDoc}
         */
        public function current() {}

        /**
         * @see \Iterator::key()
         * @see https://www.php.net/manual/en/iterator.key.php
         * {@inheritDoc}
         */
        public function key() {}

        /**
         * @return void
         * @see \Iterator::next()
         * @see https://www.php.net/manual/en/iterator.next.php
         * {@inheritDoc}
         */
        public function next() {}

        /**
         * @return void
         * @see \Iterator::rewind()
         * @see https://www.php.net/manual/en/iterator.rewind.php
         * {@inheritDoc}
         */
        public function rewind() {}

        /**
         * @return bool
         * @see \Iterator::valid()
         * @see https://www.php.net/manual/en/iterator.valid.php
         * {@inheritDoc}
         */
        public function valid() {}

        /**
         * Whether or not an offset exists.
         *
         * @return bool returns true on success or false on failure
         * @see \ArrayAccess::offsetExists()
         * @see https://www.php.net/manual/en/arrayaccess.offsetexists.php
         * {@inheritDoc}
         */
        public function offsetExists($offset) {}

        /**
         * Returns the value at specified offset.
         *
         * @see \ArrayAccess::offsetGet()
         * @see https://www.php.net/manual/en/arrayaccess.offsetget.php
         * {@inheritDoc}
         */
        public function offsetGet($offset) {}

        /**
         * Assigns a value to the specified offset.
         *
         * @return void
         * @see \ArrayAccess::offsetSet()
         * @see https://www.php.net/manual/en/arrayaccess.offsetset.php
         * {@inheritDoc}
         */
        public function offsetSet($offset, $value) {}

        /**
         * Unsets an offset.
         *
         * @return void
         * @see \ArrayAccess::offsetUnset()
         * @see https://www.php.net/manual/en/arrayaccess.offsetunset.php
         * {@inheritDoc}
         */
        public function offsetUnset($offset) {}

        /**
         * @return int
         * @see \Countable::count()
         * @see https://www.php.net/manual/en/countable.count.php
         * {@inheritDoc}
         */
        public function count() {}
    }
}

namespace Swoole {
    class Thread
    {
        public int $id;

        public function __construct(string $script_file, mixed ...$args) {}

        public function join(): bool {}

        public function joinable(): bool {}

        public function getExitStatus(): int {}

        public function isAlive(): bool {}

        public function detach(): bool {}

        public static function getArguments(): ?array {}

        public static function getId(): int {}

        public static function getInfo(): array {}

        public static function activeCount(): int {}

        public static function yield(): void {}

        public static function setName(string $name): bool {}

        public static function setAffinity(array $cpu_settings): bool {}

        public static function getAffinity(): array {}

        public static function setPriority(int $priority, int $policy = 0): bool {}

        public static function getPriority(): array {}

        public static function getNativeId(): int {}
    }
}

namespace Swoole\Thread {
    use ArrayAccess;
    use Countable;

    class ArrayList implements ArrayAccess, Countable
    {
        public function __construct(?array $array = null) {}

        public function offsetGet(mixed $key): mixed {}

        public function offsetExists(mixed $key): bool {}

        public function offsetSet(mixed $key, mixed $value): void {}

        public function offsetUnset(mixed $key): void {}

        public function find(mixed $value): int {}

        public function count(): int {}

        public function incr(mixed $key, mixed $value = 1): mixed {}

        public function decr(mixed $key, mixed $value = 1): mixed {}

        public function clean(): void {}

        public function toArray(): array {}

        public function sort(): void {}
    }
}

namespace Swoole\Thread {
    class Atomic
    {
        public function __construct(int $value = 0) {}

        public function add(int $add_value = 1): int {}

        public function sub(int $sub_value = 1): int {}

        public function get(): int {}

        public function set(int $value): void {}

        public function cmpset(int $cmp_value, int $new_value): bool {}

        public function wait(float $timeout = 1.0): bool {}

        public function wakeup(int $count = 1): bool {}
    }
}

namespace Swoole\Thread\Atomic {
    class Long
    {
        public function __construct(int $value = 0) {}

        public function add(int $add_value = 1): int {}

        public function sub(int $sub_value = 1): int {}

        public function get(): int {}

        public function set(int $value): void {}

        public function cmpset(int $cmp_value, int $new_value): bool {}
    }
}

namespace Swoole\Thread {
    class Barrier
    {
        public function __construct(int $count) {}

        public function wait(): void {}
    }
}

namespace Swoole\Thread {
    class Lock
    {
        public function __construct(int $type = SWOOLE_MUTEX) {}

        public function lock(int $operation = LOCK_EX, float $timeout = -1): bool {}

        public function unlock(): bool {}
    }
}

namespace Swoole\Thread {
    use ArrayAccess;
    use Countable;

    class Map implements ArrayAccess, Countable
    {
        public function __construct(?array $array = null) {}

        public function offsetGet(mixed $key): mixed {}

        public function offsetExists(mixed $key): bool {}

        public function offsetSet(mixed $key, mixed $value): void {}

        public function offsetUnset(mixed $key): void {}

        public function find(mixed $value): mixed {}

        public function count(): int {}

        public function keys(): array {}

        public function values(): array {}

        public function incr(mixed $key, mixed $value = 1): mixed {}

        public function decr(mixed $key, mixed $value = 1): mixed {}

        public function add(mixed $key, mixed $value): bool {}

        public function update(mixed $key, mixed $value): bool {}

        public function clean(): void {}

        public function toArray(): array {}

        public function sort(): void {}
    }
}

namespace Swoole\Thread {
    use Countable;

    class Queue implements Countable
    {
        public function __construct() {}

        public function push(mixed $value, int $notify_which = 0): void {}

        public function pop(float $wait = 0): mixed {}

        public function count(): int {}

        public function clean(): void {}
    }
}

namespace Swoole {
    class Timer
    {
        /** @return void */
        public static function set(array $settings) {}

        /** @return int */
        public static function tick(int $ms, callable $callback, ...$params) {}

        /** @return int */
        public static function after(int $ms, callable $callback, ...$params) {}

        /** @return bool */
        public static function exists(int $timer_id) {}

        /** @return array */
        public static function info(int $timer_id) {}

        /** @return array */
        public static function stats() {}

        /** @return \Swoole\timer\Iterator */
        public static function list() {}

        /** @return bool */
        public static function clear(int $timer_id) {}

        /** @return bool */
        public static function clearAll() {}
    }
}

namespace Swoole\Timer {
    /** @see https://www.php.net/ArrayIterator */
    class Iterator extends \ArrayIterator {}
}

namespace Swoole\WebSocket {
    class CloseFrame extends Frame
    {
        public $opcode = 8;
        public $code = 1000;
        public $reason = '';
    }
}

namespace Swoole\WebSocket {
    class Frame
    {
        public $fd = 0;
        public $data = '';
        public $opcode = 1;
        public $flags = 1;
        public $finish;

        public function __toString(): string {}

        public static function pack($data, $opcode = null, $flags = null) {}

        public static function unpack($data) {}
    }
}

namespace Swoole\WebSocket {
    class Server extends \Swoole\Http\Server
    {
        public function push($fd, $data, $opcode = null, $flags = null) {}

        public function disconnect($fd, $code = null, $reason = null) {}

        public function isEstablished($fd) {}

        public static function pack($data, $opcode = null, $flags = null) {}

        public static function unpack($data) {}
    }
}

namespace {
    define('SWOOLE_VERSION', '4.8.6');
    define('SWOOLE_VERSION_ID', 40806);
    define('SWOOLE_MAJOR_VERSION', 4);
    define('SWOOLE_MINOR_VERSION', 8);
    define('SWOOLE_RELEASE_VERSION', 6);
    define('SWOOLE_EXTRA_VERSION', '');
    define('SWOOLE_DEBUG', '');
    define('SWOOLE_HAVE_COMPRESSION', '1');
    define('SWOOLE_HAVE_ZLIB', '1');
    define('SWOOLE_HAVE_BROTLI', '1');
    define('SWOOLE_USE_HTTP2', '1');
    define('SWOOLE_USE_SHORTNAME', '1');
    define('SWOOLE_SOCK_TCP', 1);
    define('SWOOLE_SOCK_TCP6', 3);
    define('SWOOLE_SOCK_UDP', 2);
    define('SWOOLE_SOCK_UDP6', 4);
    define('SWOOLE_SOCK_UNIX_DGRAM', 6);
    define('SWOOLE_SOCK_UNIX_STREAM', 5);
    define('SWOOLE_TCP', 1);
    define('SWOOLE_TCP6', 3);
    define('SWOOLE_UDP', 2);
    define('SWOOLE_UDP6', 4);
    define('SWOOLE_UNIX_DGRAM', 6);
    define('SWOOLE_UNIX_STREAM', 5);
    define('SWOOLE_SOCK_SYNC', '');
    define('SWOOLE_SOCK_ASYNC', '1');
    define('SWOOLE_SYNC', 2048);
    define('SWOOLE_ASYNC', 1024);
    define('SWOOLE_KEEP', 4096);
    define('SWOOLE_SSL', 512);
    define('SWOOLE_SSLv3_METHOD', 1);
    define('SWOOLE_SSLv3_SERVER_METHOD', 2);
    define('SWOOLE_SSLv3_CLIENT_METHOD', 3);
    define('SWOOLE_TLSv1_METHOD', 6);
    define('SWOOLE_TLSv1_SERVER_METHOD', 7);
    define('SWOOLE_TLSv1_CLIENT_METHOD', 8);
    define('SWOOLE_TLSv1_1_METHOD', 9);
    define('SWOOLE_TLSv1_1_SERVER_METHOD', 10);
    define('SWOOLE_TLSv1_1_CLIENT_METHOD', 11);
    define('SWOOLE_TLSv1_2_METHOD', 12);
    define('SWOOLE_TLSv1_2_SERVER_METHOD', 13);
    define('SWOOLE_TLSv1_2_CLIENT_METHOD', 14);
    define('SWOOLE_DTLS_SERVER_METHOD', 16);
    define('SWOOLE_DTLS_CLIENT_METHOD', 15);
    define('SWOOLE_SSLv23_METHOD', 0);
    define('SWOOLE_SSLv23_SERVER_METHOD', 4);
    define('SWOOLE_SSLv23_CLIENT_METHOD', 5);
    define('SWOOLE_TLS_METHOD', 0);
    define('SWOOLE_TLS_SERVER_METHOD', 4);
    define('SWOOLE_TLS_CLIENT_METHOD', 5);
    define('SWOOLE_SSL_TLSv1', 8);
    define('SWOOLE_SSL_TLSv1_1', 16);
    define('SWOOLE_SSL_TLSv1_2', 32);
    define('SWOOLE_SSL_TLSv1_3', 64);
    define('SWOOLE_SSL_DTLS', 128);
    define('SWOOLE_SSL_SSLv2', 2);
    define('SWOOLE_EVENT_READ', 512);
    define('SWOOLE_EVENT_WRITE', 1024);
    define('SWOOLE_STRERROR_SYSTEM', 0);
    define('SWOOLE_STRERROR_GAI', 1);
    define('SWOOLE_STRERROR_DNS', 2);
    define('SWOOLE_STRERROR_SWOOLE', 9);
    define('SWOOLE_ERROR_MALLOC_FAIL', 501);
    define('SWOOLE_ERROR_SYSTEM_CALL_FAIL', 502);
    define('SWOOLE_ERROR_PHP_FATAL_ERROR', 503);
    define('SWOOLE_ERROR_NAME_TOO_LONG', 504);
    define('SWOOLE_ERROR_INVALID_PARAMS', 505);
    define('SWOOLE_ERROR_QUEUE_FULL', 506);
    define('SWOOLE_ERROR_OPERATION_NOT_SUPPORT', 507);
    define('SWOOLE_ERROR_PROTOCOL_ERROR', 508);
    define('SWOOLE_ERROR_WRONG_OPERATION', 509);
    define('SWOOLE_ERROR_FILE_NOT_EXIST', 700);
    define('SWOOLE_ERROR_FILE_TOO_LARGE', 701);
    define('SWOOLE_ERROR_FILE_EMPTY', 702);
    define('SWOOLE_ERROR_DNSLOOKUP_DUPLICATE_REQUEST', 710);
    define('SWOOLE_ERROR_DNSLOOKUP_RESOLVE_FAILED', 711);
    define('SWOOLE_ERROR_DNSLOOKUP_RESOLVE_TIMEOUT', 712);
    define('SWOOLE_ERROR_DNSLOOKUP_UNSUPPORTED', 713);
    define('SWOOLE_ERROR_DNSLOOKUP_NO_SERVER', 714);
    define('SWOOLE_ERROR_BAD_IPV6_ADDRESS', 720);
    define('SWOOLE_ERROR_UNREGISTERED_SIGNAL', 721);
    define('SWOOLE_ERROR_EVENT_SOCKET_REMOVED', 800);
    define('SWOOLE_ERROR_SESSION_CLOSED_BY_SERVER', 1001);
    define('SWOOLE_ERROR_SESSION_CLOSED_BY_CLIENT', 1002);
    define('SWOOLE_ERROR_SESSION_CLOSING', 1003);
    define('SWOOLE_ERROR_SESSION_CLOSED', 1004);
    define('SWOOLE_ERROR_SESSION_NOT_EXIST', 1005);
    define('SWOOLE_ERROR_SESSION_INVALID_ID', 1006);
    define('SWOOLE_ERROR_SESSION_DISCARD_TIMEOUT_DATA', 1007);
    define('SWOOLE_ERROR_SESSION_DISCARD_DATA', 1008);
    define('SWOOLE_ERROR_OUTPUT_BUFFER_OVERFLOW', 1009);
    define('SWOOLE_ERROR_OUTPUT_SEND_YIELD', 1010);
    define('SWOOLE_ERROR_SSL_NOT_READY', 1011);
    define('SWOOLE_ERROR_SSL_CANNOT_USE_SENFILE', 1012);
    define('SWOOLE_ERROR_SSL_EMPTY_PEER_CERTIFICATE', 1013);
    define('SWOOLE_ERROR_SSL_VERIFY_FAILED', 1014);
    define('SWOOLE_ERROR_SSL_BAD_CLIENT', 1015);
    define('SWOOLE_ERROR_SSL_BAD_PROTOCOL', 1016);
    define('SWOOLE_ERROR_SSL_RESET', 1017);
    define('SWOOLE_ERROR_SSL_HANDSHAKE_FAILED', 1018);
    define('SWOOLE_ERROR_PACKAGE_LENGTH_TOO_LARGE', 1201);
    define('SWOOLE_ERROR_PACKAGE_LENGTH_NOT_FOUND', 1202);
    define('SWOOLE_ERROR_DATA_LENGTH_TOO_LARGE', 1203);
    define('SWOOLE_ERROR_PACKAGE_MALFORMED_DATA', 1204);
    define('SWOOLE_ERROR_TASK_PACKAGE_TOO_BIG', 2001);
    define('SWOOLE_ERROR_TASK_DISPATCH_FAIL', 2002);
    define('SWOOLE_ERROR_TASK_TIMEOUT', 2003);
    define('SWOOLE_ERROR_HTTP2_STREAM_ID_TOO_BIG', 3001);
    define('SWOOLE_ERROR_HTTP2_STREAM_NO_HEADER', 3002);
    define('SWOOLE_ERROR_HTTP2_STREAM_NOT_FOUND', 3003);
    define('SWOOLE_ERROR_HTTP2_STREAM_IGNORE', 3004);
    define('SWOOLE_ERROR_AIO_BAD_REQUEST', 4001);
    define('SWOOLE_ERROR_AIO_CANCELED', 4002);
    define('SWOOLE_ERROR_AIO_TIMEOUT', 4003);
    define('SWOOLE_ERROR_CLIENT_NO_CONNECTION', 5001);
    define('SWOOLE_ERROR_SOCKET_CLOSED', 6001);
    define('SWOOLE_ERROR_SOCKET_POLL_TIMEOUT', 6002);
    define('SWOOLE_ERROR_SOCKS5_UNSUPPORT_VERSION', 7001);
    define('SWOOLE_ERROR_SOCKS5_UNSUPPORT_METHOD', 7002);
    define('SWOOLE_ERROR_SOCKS5_AUTH_FAILED', 7003);
    define('SWOOLE_ERROR_SOCKS5_SERVER_ERROR', 7004);
    define('SWOOLE_ERROR_SOCKS5_HANDSHAKE_FAILED', 7005);
    define('SWOOLE_ERROR_HTTP_PROXY_HANDSHAKE_ERROR', 7101);
    define('SWOOLE_ERROR_HTTP_INVALID_PROTOCOL', 7102);
    define('SWOOLE_ERROR_HTTP_PROXY_HANDSHAKE_FAILED', 7103);
    define('SWOOLE_ERROR_HTTP_PROXY_BAD_RESPONSE', 7104);
    define('SWOOLE_ERROR_WEBSOCKET_BAD_CLIENT', 8501);
    define('SWOOLE_ERROR_WEBSOCKET_BAD_OPCODE', 8502);
    define('SWOOLE_ERROR_WEBSOCKET_UNCONNECTED', 8503);
    define('SWOOLE_ERROR_WEBSOCKET_HANDSHAKE_FAILED', 8504);
    define('SWOOLE_ERROR_WEBSOCKET_PACK_FAILED', 8505);
    define('SWOOLE_ERROR_WEBSOCKET_UNPACK_FAILED', 8506);
    define('SWOOLE_ERROR_WEBSOCKET_INCOMPLETE_PACKET', 8507);
    define('SWOOLE_ERROR_SERVER_MUST_CREATED_BEFORE_CLIENT', 9001);
    define('SWOOLE_ERROR_SERVER_TOO_MANY_SOCKET', 9002);
    define('SWOOLE_ERROR_SERVER_WORKER_TERMINATED', 9003);
    define('SWOOLE_ERROR_SERVER_INVALID_LISTEN_PORT', 9004);
    define('SWOOLE_ERROR_SERVER_TOO_MANY_LISTEN_PORT', 9005);
    define('SWOOLE_ERROR_SERVER_PIPE_BUFFER_FULL', 9006);
    define('SWOOLE_ERROR_SERVER_NO_IDLE_WORKER', 9007);
    define('SWOOLE_ERROR_SERVER_ONLY_START_ONE', 9008);
    define('SWOOLE_ERROR_SERVER_SEND_IN_MASTER', 9009);
    define('SWOOLE_ERROR_SERVER_INVALID_REQUEST', 9010);
    define('SWOOLE_ERROR_SERVER_CONNECT_FAIL', 9011);
    define('SWOOLE_ERROR_SERVER_INVALID_COMMAND', 9012);
    define('SWOOLE_ERROR_SERVER_WORKER_EXIT_TIMEOUT', 9101);
    define('SWOOLE_ERROR_SERVER_WORKER_ABNORMAL_PIPE_DATA', 9102);
    define('SWOOLE_ERROR_SERVER_WORKER_UNPROCESSED_DATA', 9103);
    define('SWOOLE_ERROR_CO_OUT_OF_COROUTINE', 10001);
    define('SWOOLE_ERROR_CO_HAS_BEEN_BOUND', 10002);
    define('SWOOLE_ERROR_CO_HAS_BEEN_DISCARDED', 10003);
    define('SWOOLE_ERROR_CO_MUTEX_DOUBLE_UNLOCK', 10004);
    define('SWOOLE_ERROR_CO_BLOCK_OBJECT_LOCKED', 10005);
    define('SWOOLE_ERROR_CO_BLOCK_OBJECT_WAITING', 10006);
    define('SWOOLE_ERROR_CO_YIELD_FAILED', 10007);
    define('SWOOLE_ERROR_CO_GETCONTEXT_FAILED', 10008);
    define('SWOOLE_ERROR_CO_SWAPCONTEXT_FAILED', 10009);
    define('SWOOLE_ERROR_CO_MAKECONTEXT_FAILED', 10010);
    define('SWOOLE_ERROR_CO_IOCPINIT_FAILED', 10011);
    define('SWOOLE_ERROR_CO_PROTECT_STACK_FAILED', 10012);
    define('SWOOLE_ERROR_CO_STD_THREAD_LINK_ERROR', 10013);
    define('SWOOLE_ERROR_CO_DISABLED_MULTI_THREAD', 10014);
    define('SWOOLE_ERROR_CO_CANNOT_CANCEL', 10015);
    define('SWOOLE_ERROR_CO_NOT_EXISTS', 10016);
    define('SWOOLE_ERROR_CO_CANCELED', 10017);
    define('SWOOLE_ERROR_CO_TIMEDOUT', 10018);
    define('SWOOLE_TRACE_SERVER', 2);
    define('SWOOLE_TRACE_CLIENT', 4);
    define('SWOOLE_TRACE_BUFFER', 8);
    define('SWOOLE_TRACE_CONN', 16);
    define('SWOOLE_TRACE_EVENT', 32);
    define('SWOOLE_TRACE_WORKER', 64);
    define('SWOOLE_TRACE_MEMORY', 128);
    define('SWOOLE_TRACE_REACTOR', 256);
    define('SWOOLE_TRACE_PHP', 512);
    define('SWOOLE_TRACE_HTTP', 1024);
    define('SWOOLE_TRACE_HTTP2', 2048);
    define('SWOOLE_TRACE_EOF_PROTOCOL', 4096);
    define('SWOOLE_TRACE_LENGTH_PROTOCOL', 8192);
    define('SWOOLE_TRACE_CLOSE', 16384);
    define('SWOOLE_TRACE_WEBSOCKET', 32768);
    define('SWOOLE_TRACE_REDIS_CLIENT', 65536);
    define('SWOOLE_TRACE_MYSQL_CLIENT', 131072);
    define('SWOOLE_TRACE_HTTP_CLIENT', 262144);
    define('SWOOLE_TRACE_AIO', 524288);
    define('SWOOLE_TRACE_SSL', 1048576);
    define('SWOOLE_TRACE_NORMAL', 2097152);
    define('SWOOLE_TRACE_CHANNEL', 4194304);
    define('SWOOLE_TRACE_TIMER', 8388608);
    define('SWOOLE_TRACE_SOCKET', 16777216);
    define('SWOOLE_TRACE_COROUTINE', 33554432);
    define('SWOOLE_TRACE_CONTEXT', 67108864);
    define('SWOOLE_TRACE_CO_HTTP_SERVER', 134217728);
    define('SWOOLE_TRACE_TABLE', 268435456);
    define('SWOOLE_TRACE_CO_CURL', 536870912);
    define('SWOOLE_TRACE_CARES', 1073741824);
    define('SWOOLE_TRACE_ALL', 9223372036854775807);
    define('SWOOLE_LOG_DEBUG', 0);
    define('SWOOLE_LOG_TRACE', 1);
    define('SWOOLE_LOG_INFO', 2);
    define('SWOOLE_LOG_NOTICE', 3);
    define('SWOOLE_LOG_WARNING', 4);
    define('SWOOLE_LOG_ERROR', 5);
    define('SWOOLE_LOG_NONE', 6);
    define('SWOOLE_LOG_ROTATION_SINGLE', 0);
    define('SWOOLE_LOG_ROTATION_MONTHLY', 1);
    define('SWOOLE_LOG_ROTATION_DAILY', 2);
    define('SWOOLE_LOG_ROTATION_HOURLY', 3);
    define('SWOOLE_LOG_ROTATION_EVERY_MINUTE', 4);
    define('SWOOLE_IPC_NONE', 0);
    define('SWOOLE_IPC_UNIXSOCK', 1);
    define('SWOOLE_IPC_SOCKET', 3);
    define('SWOOLE_IOV_MAX', 1024);
    define('SWOOLE_FILELOCK', 2);
    define('SWOOLE_MUTEX', 3);
    define('SWOOLE_SEM', 4);
    define('SWOOLE_RWLOCK', 1);
    define('SWOOLE_SPINLOCK', 5);
    define('SWOOLE_TIMER_MIN_MS', 1);
    define('SWOOLE_TIMER_MIN_SEC', 0.001);
    define('SWOOLE_TIMER_MAX_MS', 9223372036854775807);
    define('SWOOLE_TIMER_MAX_SEC', 9.2233720368548E+15);
    define('SWOOLE_DEFAULT_MAX_CORO_NUM', 100000);
    define('SWOOLE_CORO_MAX_NUM_LIMIT', 9223372036854775807);
    define('SWOOLE_CORO_INIT', 0);
    define('SWOOLE_CORO_WAITING', 1);
    define('SWOOLE_CORO_RUNNING', 2);
    define('SWOOLE_CORO_END', 3);
    define('SWOOLE_EXIT_IN_COROUTINE', 2);
    define('SWOOLE_EXIT_IN_SERVER', 4);
    define('SWOOLE_CHANNEL_OK', 0);
    define('SWOOLE_CHANNEL_TIMEOUT', -1);
    define('SWOOLE_CHANNEL_CLOSED', -2);
    define('SWOOLE_CHANNEL_CANCELED', -3);
    define('SWOOLE_HOOK_TCP', 2);
    define('SWOOLE_HOOK_UDP', 4);
    define('SWOOLE_HOOK_UNIX', 8);
    define('SWOOLE_HOOK_UDG', 16);
    define('SWOOLE_HOOK_SSL', 32);
    define('SWOOLE_HOOK_TLS', 64);
    define('SWOOLE_HOOK_STREAM_FUNCTION', 128);
    define('SWOOLE_HOOK_STREAM_SELECT', 128);
    define('SWOOLE_HOOK_FILE', 256);
    define('SWOOLE_HOOK_STDIO', 32768);
    define('SWOOLE_HOOK_SLEEP', 512);
    define('SWOOLE_HOOK_PROC', 1024);
    define('SWOOLE_HOOK_CURL', 2048);
    define('SWOOLE_HOOK_NATIVE_CURL', 4096);
    define('SWOOLE_HOOK_BLOCKING_FUNCTION', 8192);
    define('SWOOLE_HOOK_SOCKETS', 16384);
    define('SWOOLE_HOOK_ALL', 2147481599);
    define('SOCKET_ECANCELED', 125);
    define('SWOOLE_HTTP_CLIENT_ESTATUS_CONNECT_FAILED', -1);
    define('SWOOLE_HTTP_CLIENT_ESTATUS_REQUEST_TIMEOUT', -2);
    define('SWOOLE_HTTP_CLIENT_ESTATUS_SERVER_RESET', -3);
    define('SWOOLE_HTTP_CLIENT_ESTATUS_SEND_FAILED', -4);
    define('SWOOLE_MYSQLND_CR_UNKNOWN_ERROR', 2000);
    define('SWOOLE_MYSQLND_CR_CONNECTION_ERROR', 2002);
    define('SWOOLE_MYSQLND_CR_SERVER_GONE_ERROR', 2006);
    define('SWOOLE_MYSQLND_CR_OUT_OF_MEMORY', 2008);
    define('SWOOLE_MYSQLND_CR_SERVER_LOST', 2013);
    define('SWOOLE_MYSQLND_CR_COMMANDS_OUT_OF_SYNC', 2014);
    define('SWOOLE_MYSQLND_CR_CANT_FIND_CHARSET', 2019);
    define('SWOOLE_MYSQLND_CR_MALFORMED_PACKET', 2027);
    define('SWOOLE_MYSQLND_CR_NOT_IMPLEMENTED', 2054);
    define('SWOOLE_MYSQLND_CR_NO_PREPARE_STMT', 2030);
    define('SWOOLE_MYSQLND_CR_PARAMS_NOT_BOUND', 2031);
    define('SWOOLE_MYSQLND_CR_INVALID_PARAMETER_NO', 2034);
    define('SWOOLE_MYSQLND_CR_INVALID_BUFFER_USE', 2035);
    define('SWOOLE_REDIS_MODE_MULTI', 0);
    define('SWOOLE_REDIS_MODE_PIPELINE', 1);
    define('SWOOLE_REDIS_TYPE_NOT_FOUND', 0);
    define('SWOOLE_REDIS_TYPE_STRING', 1);
    define('SWOOLE_REDIS_TYPE_SET', 2);
    define('SWOOLE_REDIS_TYPE_LIST', 3);
    define('SWOOLE_REDIS_TYPE_ZSET', 4);
    define('SWOOLE_REDIS_TYPE_HASH', 5);
    define('SWOOLE_REDIS_ERR_IO', 1);
    define('SWOOLE_REDIS_ERR_OTHER', 2);
    define('SWOOLE_REDIS_ERR_EOF', 3);
    define('SWOOLE_REDIS_ERR_PROTOCOL', 4);
    define('SWOOLE_REDIS_ERR_OOM', 5);
    define('SWOOLE_REDIS_ERR_CLOSED', 6);
    define('SWOOLE_REDIS_ERR_NOAUTH', 7);
    define('SWOOLE_REDIS_ERR_ALLOC', 8);
    define('SWOOLE_HTTP2_TYPE_DATA', 0);
    define('SWOOLE_HTTP2_TYPE_HEADERS', 1);
    define('SWOOLE_HTTP2_TYPE_PRIORITY', 2);
    define('SWOOLE_HTTP2_TYPE_RST_STREAM', 3);
    define('SWOOLE_HTTP2_TYPE_SETTINGS', 4);
    define('SWOOLE_HTTP2_TYPE_PUSH_PROMISE', 5);
    define('SWOOLE_HTTP2_TYPE_PING', 6);
    define('SWOOLE_HTTP2_TYPE_GOAWAY', 7);
    define('SWOOLE_HTTP2_TYPE_WINDOW_UPDATE', 8);
    define('SWOOLE_HTTP2_TYPE_CONTINUATION', 9);
    define('SWOOLE_HTTP2_ERROR_NO_ERROR', 0);
    define('SWOOLE_HTTP2_ERROR_PROTOCOL_ERROR', 1);
    define('SWOOLE_HTTP2_ERROR_INTERNAL_ERROR', 2);
    define('SWOOLE_HTTP2_ERROR_FLOW_CONTROL_ERROR', 3);
    define('SWOOLE_HTTP2_ERROR_SETTINGS_TIMEOUT', 4);
    define('SWOOLE_HTTP2_ERROR_STREAM_CLOSED', 5);
    define('SWOOLE_HTTP2_ERROR_FRAME_SIZE_ERROR', 6);
    define('SWOOLE_HTTP2_ERROR_REFUSED_STREAM', 7);
    define('SWOOLE_HTTP2_ERROR_CANCEL', 8);
    define('SWOOLE_HTTP2_ERROR_COMPRESSION_ERROR', 9);
    define('SWOOLE_HTTP2_ERROR_CONNECT_ERROR', 10);
    define('SWOOLE_HTTP2_ERROR_ENHANCE_YOUR_CALM', 11);
    define('SWOOLE_HTTP2_ERROR_INADEQUATE_SECURITY', 12);
    define('SWOOLE_BASE', 1);
    define('SWOOLE_PROCESS', 2);
    define('SWOOLE_IPC_UNSOCK', 1);
    define('SWOOLE_IPC_MSGQUEUE', 2);
    define('SWOOLE_IPC_PREEMPTIVE', 3);
    define('SWOOLE_SERVER_COMMAND_MASTER', 2);
    define('SWOOLE_SERVER_COMMAND_MANAGER', 32);
    define('SWOOLE_SERVER_COMMAND_REACTOR_THREAD', 4);
    define('SWOOLE_SERVER_COMMAND_EVENT_WORKER', 8);
    define('SWOOLE_SERVER_COMMAND_WORKER', 8);
    define('SWOOLE_SERVER_COMMAND_TASK_WORKER', 16);
    define('SWOOLE_DISPATCH_ROUND', 1);
    define('SWOOLE_DISPATCH_FDMOD', 2);
    define('SWOOLE_DISPATCH_IDLE_WORKER', 3);
    define('SWOOLE_DISPATCH_IPMOD', 4);
    define('SWOOLE_DISPATCH_UIDMOD', 5);
    define('SWOOLE_DISPATCH_USERFUNC', 6);
    define('SWOOLE_DISPATCH_STREAM', 7);
    define('SWOOLE_DISPATCH_CO_CONN_LB', 8);
    define('SWOOLE_DISPATCH_CO_REQ_LB', 9);
    define('SWOOLE_DISPATCH_RESULT_DISCARD_PACKET', -1);
    define('SWOOLE_DISPATCH_RESULT_CLOSE_CONNECTION', -2);
    define('SWOOLE_DISPATCH_RESULT_USERFUNC_FALLBACK', -3);
    define('SWOOLE_TASK_TMPFILE', 1);
    define('SWOOLE_TASK_SERIALIZE', 2);
    define('SWOOLE_TASK_NONBLOCK', 4);
    define('SWOOLE_TASK_CALLBACK', 8);
    define('SWOOLE_TASK_WAITALL', 16);
    define('SWOOLE_TASK_COROUTINE', 32);
    define('SWOOLE_TASK_PEEK', 64);
    define('SWOOLE_TASK_NOREPLY', 128);
    define('SWOOLE_WORKER_BUSY', 1);
    define('SWOOLE_WORKER_IDLE', 2);
    define('SWOOLE_WORKER_EXIT', 3);
    define('SWOOLE_WEBSOCKET_STATUS_CONNECTION', 1);
    define('SWOOLE_WEBSOCKET_STATUS_HANDSHAKE', 2);
    define('SWOOLE_WEBSOCKET_STATUS_ACTIVE', 3);
    define('SWOOLE_WEBSOCKET_STATUS_CLOSING', 4);
    define('SWOOLE_WEBSOCKET_OPCODE_CONTINUATION', 0);
    define('SWOOLE_WEBSOCKET_OPCODE_TEXT', 1);
    define('SWOOLE_WEBSOCKET_OPCODE_BINARY', 2);
    define('SWOOLE_WEBSOCKET_OPCODE_CLOSE', 8);
    define('SWOOLE_WEBSOCKET_OPCODE_PING', 9);
    define('SWOOLE_WEBSOCKET_OPCODE_PONG', 10);
    define('SWOOLE_WEBSOCKET_FLAG_FIN', 1);
    define('SWOOLE_WEBSOCKET_FLAG_RSV1', 4);
    define('SWOOLE_WEBSOCKET_FLAG_RSV2', 8);
    define('SWOOLE_WEBSOCKET_FLAG_RSV3', 16);
    define('SWOOLE_WEBSOCKET_FLAG_MASK', 32);
    define('SWOOLE_WEBSOCKET_FLAG_COMPRESS', 2);
    define('SWOOLE_WEBSOCKET_CLOSE_NORMAL', 1000);
    define('SWOOLE_WEBSOCKET_CLOSE_GOING_AWAY', 1001);
    define('SWOOLE_WEBSOCKET_CLOSE_PROTOCOL_ERROR', 1002);
    define('SWOOLE_WEBSOCKET_CLOSE_DATA_ERROR', 1003);
    define('SWOOLE_WEBSOCKET_CLOSE_STATUS_ERROR', 1005);
    define('SWOOLE_WEBSOCKET_CLOSE_ABNORMAL', 1006);
    define('SWOOLE_WEBSOCKET_CLOSE_MESSAGE_ERROR', 1007);
    define('SWOOLE_WEBSOCKET_CLOSE_POLICY_ERROR', 1008);
    define('SWOOLE_WEBSOCKET_CLOSE_MESSAGE_TOO_BIG', 1009);
    define('SWOOLE_WEBSOCKET_CLOSE_EXTENSION_MISSING', 1010);
    define('SWOOLE_WEBSOCKET_CLOSE_SERVER_ERROR', 1011);
    define('SWOOLE_WEBSOCKET_CLOSE_TLS', 1015);
    define('WEBSOCKET_STATUS_CONNECTION', 1);
    define('WEBSOCKET_STATUS_HANDSHAKE', 2);
    define('WEBSOCKET_STATUS_FRAME', 3);
    define('WEBSOCKET_STATUS_ACTIVE', 3);
    define('WEBSOCKET_STATUS_CLOSING', 4);
    define('WEBSOCKET_OPCODE_CONTINUATION', 0);
    define('WEBSOCKET_OPCODE_TEXT', 1);
    define('WEBSOCKET_OPCODE_BINARY', 2);
    define('WEBSOCKET_OPCODE_CLOSE', 8);
    define('WEBSOCKET_OPCODE_PING', 9);
    define('WEBSOCKET_OPCODE_PONG', 10);
    define('WEBSOCKET_CLOSE_NORMAL', 1000);
    define('WEBSOCKET_CLOSE_GOING_AWAY', 1001);
    define('WEBSOCKET_CLOSE_PROTOCOL_ERROR', 1002);
    define('WEBSOCKET_CLOSE_DATA_ERROR', 1003);
    define('WEBSOCKET_CLOSE_STATUS_ERROR', 1005);
    define('WEBSOCKET_CLOSE_ABNORMAL', 1006);
    define('WEBSOCKET_CLOSE_MESSAGE_ERROR', 1007);
    define('WEBSOCKET_CLOSE_POLICY_ERROR', 1008);
    define('WEBSOCKET_CLOSE_MESSAGE_TOO_BIG', 1009);
    define('WEBSOCKET_CLOSE_EXTENSION_MISSING', 1010);
    define('WEBSOCKET_CLOSE_SERVER_ERROR', 1011);
    define('WEBSOCKET_CLOSE_TLS', 1015);
}

namespace Swoole\Coroutine {
    function run(callable $func, mixed ...$params): bool {}

    function go(callable $func, mixed ...$params): int|false {}
}

namespace {
    /**
     * Gets the current Swoole version. This information is also available in the predefined constant SWOOLE_VERSION.
     *
     * @return string returns a string containing the version of Swoole
     */
    function swoole_version() {}

    /**
     * Gets the number of CPU cores.
     *
     * @return int returns the number of CPU cores
     */
    function swoole_cpu_num() {}

    function swoole_last_error() {}

    /**
     * @param $domain_name[required]
     * @param $timeout[optional]
     * @param $type[optional]
     */
    function swoole_async_dns_lookup_coro($domain_name, $timeout = null, $type = null) {}

    /** @param $settings[required] */
    function swoole_async_set($settings) {}

    /** @return int|false */
    function swoole_coroutine_create(callable $func, ...$params) {}

    /**
     * Defers the execution of a callback function until the surrounding function of a coroutine returns.
     *
     * @return void
     * @example
     * <pre>
     * swoole_coroutine_create(function () {  // The surrounding function of a coroutine.
     *   echo '1';
     *   swoole_coroutine_defer(function () { // The callback function to be deferred.
     *     echo '3';
     *   });
     *   echo '2';
     * });
     * <pre>
     */
    function swoole_coroutine_defer(callable $callback) {}

    /**
     * @param $domain[required]
     * @param $type[required]
     * @param $protocol[required]
     */
    function swoole_coroutine_socketpair($domain, $type, $protocol) {}

    /**
     * @param $count[optional]
     * @param $sleep_time[optional]
     */
    function swoole_test_kernel_coroutine($count = null, $sleep_time = null) {}

    /**
     * @param $read_array[required]
     * @param $write_array[required]
     * @param $error_array[required]
     * @param $timeout[optional]
     */
    function swoole_client_select(&$read_array, &$write_array, &$error_array, $timeout = null) {}

    /**
     * @param $read_array[required]
     * @param $write_array[required]
     * @param $error_array[required]
     * @param $timeout[optional]
     */
    function swoole_select(&$read_array, &$write_array, &$error_array, $timeout = null) {}

    /** @param $process_name[required] */
    function swoole_set_process_name($process_name) {}

    function swoole_get_local_ip() {}

    function swoole_get_local_mac() {}

    /**
     * @param $errno[required]
     * @param $error_type[optional]
     */
    function swoole_strerror($errno, $error_type = null) {}

    function swoole_errno() {}

    function swoole_clear_error() {}

    /** @return void */
    function swoole_error_log(int $level, string $msg) {}

    /**
     * @return void
     * @since 4.8.1
     */
    function swoole_error_log_ex(int $level, int $error, string $msg) {}

    /**
     * @return void
     * @since 4.8.1
     */
    function swoole_ignore_error(int $error) {}

    /**
     * @param $data[required]
     * @param $type[optional]
     */
    function swoole_hashcode($data, $type = null) {}

    /**
     * @param $suffix[required]
     * @param $mime_type[required]
     */
    function swoole_mime_type_add($suffix, $mime_type) {}

    /**
     * @param $suffix[required]
     * @param $mime_type[required]
     */
    function swoole_mime_type_set($suffix, $mime_type) {}

    /** @param $suffix[required] */
    function swoole_mime_type_delete($suffix) {}

    /** @param $filename[required] */
    function swoole_mime_type_get($filename) {}

    /** @param $filename[required] */
    function swoole_get_mime_type($filename) {}

    /** @param $filename[required] */
    function swoole_mime_type_exists($filename) {}

    function swoole_mime_type_list() {}

    function swoole_clear_dns_cache() {}

    /**
     * @param $str[required]
     * @param $offset[required]
     * @param $length[optional]
     * @param $options[optional]
     */
    function swoole_substr_unserialize($str, $offset, $length = null, $options = null) {}

    /**
     * @param $json[required]
     * @param $offset[required]
     * @param $length[optional]
     * @param $associative[optional]
     * @param $depth[optional]
     * @param $flags[optional]
     */
    function swoole_substr_json_decode(
        $json,
        $offset,
        $length = null,
        $associative = null,
        $depth = null,
        $flags = null,
    ) {}

    function swoole_internal_call_user_shutdown_begin() {}

    /**
     * Get all PHP objects of current call stack.
     *
     * @return array|false Return an array of objects back; return FALSE when no objects exist or when error happens.
     * @since 4.8.1
     */
    function swoole_get_objects() {}

    /**
     * Get status information of current call stack.
     *
     * @return array The array contains two fields: "object_num" (# of objects) and "resource_num" (# of resources).
     * @since 4.8.1
     */
    function swoole_get_vm_status() {}

    /**
     * @return array|false Return the specified object back; return FALSE when no object found or when error happens.
     * @since 4.8.1
     */
    function swoole_get_object_by_handle(int $handle) {}

    /**
     * This function is an alias of function swoole_coroutine_create(); it's available only when directive
     * "swoole.use_shortname" is not explicitly turned off.
     *
     * @return int|false
     * @see swoole_coroutine_create()
     */
    function go(callable $func, ...$params) {}

    /**
     * Defers the execution of a callback function until the surrounding function of a coroutine returns.
     *
     * This function is an alias of function swoole_coroutine_defer(); it's available only when directive
     * "swoole.use_shortname" is not explicitly turned off.
     *
     * @return void
     * @see swoole_coroutine_defer()
     *
     * @example
     * <pre>
     * go(function () {      // The surrounding function of a coroutine.
     *   echo '1';
     *   defer(function () { // The callback function to be deferred.
     *     echo '3';
     *   });
     *   echo '2';
     * });
     * <pre>
     */
    function defer(callable $callback) {}

    /**
     * @param $fd[required]
     * @param $read_callback[required]
     * @param $write_callback[optional]
     * @param $events[optional]
     */
    function swoole_event_add($fd, $read_callback, $write_callback = null, $events = null) {}

    /** @param $fd[required] */
    function swoole_event_del($fd) {}

    /**
     * @param $fd[required]
     * @param $read_callback[optional]
     * @param $write_callback[optional]
     * @param $events[optional]
     */
    function swoole_event_set($fd, $read_callback = null, $write_callback = null, $events = null) {}

    /**
     * @param $fd[required]
     * @param $events[optional]
     */
    function swoole_event_isset($fd, $events = null) {}

    function swoole_event_dispatch() {}

    /**
     * This function is an alias of method \Swoole\Event::defer().
     *
     * @return true
     * @see \Swoole\Event::defer()
     */
    function swoole_event_defer(callable $callback) {}

    /**
     * @param $callback[required]
     * @param $before[optional]
     */
    function swoole_event_cycle($callback, $before = null) {}

    /**
     * @param $fd[required]
     * @param $data[required]
     */
    function swoole_event_write($fd, $data) {}

    function swoole_event_wait() {}

    function swoole_event_exit() {}

    /**
     * This function is an alias of method \Swoole\Timer::set().
     *
     * @return void
     * @see \Swoole\Timer::set()
     */
    function swoole_timer_set(array $settings) {}

    /**
     * This function is an alias of method \Swoole\Timer::after().
     *
     * @return int
     * @see \Swoole\Timer::after()
     */
    function swoole_timer_after(int $ms, callable $callback, ...$params) {}

    /**
     * This function is an alias of method \Swoole\Timer::tick().
     *
     * @return int
     * @see \Swoole\Timer::tick()
     */
    function swoole_timer_tick(int $ms, callable $callback, ...$params) {}

    /**
     * This function is an alias of method \Swoole\Timer::exists().
     *
     * @return bool
     * @see \Swoole\Timer::exists()
     */
    function swoole_timer_exists(int $timer_id) {}

    /**
     * This function is an alias of method \Swoole\Timer::info().
     *
     * @return array
     * @see \Swoole\Timer::info()
     */
    function swoole_timer_info(int $timer_id) {}

    /**
     * This function is an alias of method \Swoole\Timer::stats().
     *
     * @return array
     * @see \Swoole\Timer::stats()
     */
    function swoole_timer_stats() {}

    /**
     * This function is an alias of method \Swoole\Timer::list().
     *
     * @return \Swoole\timer\Iterator
     * @see \Swoole\Timer::list()
     */
    function swoole_timer_list() {}

    /**
     * This function is an alias of method \Swoole\Timer::clear().
     *
     * @return bool
     * @see \Swoole\Timer::clear()
     */
    function swoole_timer_clear(int $timer_id) {}

    /**
     * This function is an alias of method \Swoole\Timer::clearAll().
     *
     * @return bool
     * @see \Swoole\Timer::clearAll()
     */
    function swoole_timer_clear_all() {}
}

namespace {
    class_alias(Swoole\Coroutine\Channel::class, Co\Channel::class);
    class_alias(Swoole\Coroutine\Client::class, Co\Client::class);
    class_alias(Swoole\Coroutine\Context::class, Co\Context::class);
    class_alias(Swoole\Coroutine\Curl\Exception::class, Co\Curl\Exception::class);
    class_alias(Swoole\Coroutine\Http2\Client::class, Co\Http2\Client::class);
    class_alias(Swoole\Coroutine\Http2\Client\Exception::class, Co\Http2\Client\Exception::class);
    class_alias(Swoole\Coroutine\Http\Client::class, Co\Http\Client::class);
    class_alias(Swoole\Coroutine\Http\Client\Exception::class, Co\Http\Client\Exception::class);
    class_alias(Swoole\Coroutine\Http\Server::class, Co\Http\Server::class);
    class_alias(Swoole\Coroutine\Iterator::class, Co\Iterator::class);
    class_alias(Swoole\Coroutine\MySQL::class, Co\MySQL::class);
    class_alias(Swoole\Coroutine\MySQL\Exception::class, Co\MySQL\Exception::class);
    class_alias(Swoole\Coroutine\MySQL\Statement::class, Co\MySQL\Statement::class);
    class_alias(Swoole\Coroutine\Redis::class, Co\Redis::class);
    class_alias(Swoole\Coroutine\Scheduler::class, Co\Scheduler::class);
    class_alias(Swoole\Coroutine\Socket::class, Co\Socket::class);
    class_alias(Swoole\Coroutine\Socket\Exception::class, Co\Socket\Exception::class);
    class_alias(Swoole\Coroutine\System::class, Co\System::class);

    class_alias(Swoole\Atomic::class, swoole_atomic::class);
    class_alias(Swoole\Atomic\Long::class, swoole_atomic_long::class);
    class_alias(Swoole\Client::class, swoole_client::class);
    class_alias(Swoole\Connection\Iterator::class, swoole_connection_iterator::class);
    class_alias(Swoole\Coroutine::class, co::class);
    class_alias(Swoole\Coroutine\Channel::class, chan::class);
    class_alias(Swoole\Error::class, swoole_error::class);
    class_alias(Swoole\Event::class, swoole_event::class);
    class_alias(Swoole\Exception::class, swoole_exception::class);
    class_alias(Swoole\Http2\Request::class, swoole_http2_request::class);
    class_alias(Swoole\Http2\Response::class, swoole_http2_response::class);
    class_alias(Swoole\Http\Request::class, swoole_http_request::class);
    class_alias(Swoole\Http\Response::class, swoole_http_response::class);
    class_alias(Swoole\Http\Server::class, swoole_http_server::class);
    class_alias(Swoole\Lock::class, swoole_lock::class);
    class_alias(Swoole\Process::class, swoole_process::class);
    class_alias(Swoole\Process\Pool::class, swoole_process_pool::class);
    class_alias(Swoole\Redis\Server::class, swoole_redis_server::class);
    class_alias(Swoole\Runtime::class, swoole_runtime::class);
    class_alias(Swoole\Server::class, swoole_server::class);
    class_alias(Swoole\Server\Port::class, swoole_server_port::class);
    class_alias(Swoole\Server\Task::class, swoole_server_task::class);
    class_alias(Swoole\Table::class, swoole_table::class);
    class_alias(Swoole\Timer::class, swoole_timer::class);
    class_alias(Swoole\Timer\Iterator::class, swoole_timer_iterator::class);
    class_alias(Swoole\Websocket\Closeframe::class, swoole_websocket_closeframe::class);
    class_alias(Swoole\Websocket\Frame::class, swoole_websocket_frame::class);
    class_alias(Swoole\Websocket\Server::class, swoole_websocket_server::class);
}
