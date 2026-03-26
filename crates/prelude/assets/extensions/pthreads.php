<?php

class Pool
{
    /**
     * @var int
     */
    protected $size;

    /**
     * @var string
     */
    protected $class;

    /**
     * @var array
     */
    protected $ctor;

    /**
     * @var array
     */
    protected $workers;

    /**
     * @var int
     */
    protected $last;

    public function __construct(int $size, string $class = 'Worker', array $ctor = []) {}

    /**
     * @return int
     */
    public function collect(?callable $collector = null) {}

    /**
     * @return void
     */
    public function resize(int $size) {}

    /**
     * @return void
     */
    public function shutdown() {}

    /**
     * @return int
     */
    public function submit(Threaded $task) {}

    /**
     * @return int
     */
    public function submitTo(int $worker, Threaded $task) {}
}

class Threaded implements Collectable, Traversable, Countable, ArrayAccess
{
    /**
     * @var Worker
     */
    protected $worker;

    /**
     * @return void
     */
    public function addRef() {}

    /**
     * @param int $size
     * @param bool $preserve
     *
     * @return array
     */
    public function chunk($size, $preserve = false) {}

    /**
     * @return int
     */
    public function count() {}

    /**
     * @return void
     */
    public function delRef() {}

    /**
     * @param string $class
     *
     * @return bool
     */
    public static function extend($class) {}

    /**
     * @return int
     */
    public function getRefCount() {}

    /**
     * @return bool
     */
    public function isRunning() {}

    public function isGarbage(): bool {}

    /**
     * @return bool
     */
    public function isTerminated() {}

    /**
     * @var bool $overwrite
     *
     * @return bool
     */
    public function merge($from, $overwrite = true) {}

    /**
     * @return bool
     */
    public function notify() {}

    /**
     * @return bool
     */
    public function notifyOne() {}

    public function pop() {}

    /**
     * @return void
     */
    public function run() {}

    public function shift() {}

    public function synchronized(Closure $block, ...$_) {}

    /**
     * @return bool
     */
    public function wait(int $timeout = 0) {}

    public function offsetExists($offset) {}

    public function offsetGet($offset) {}

    public function offsetSet($offset, $value) {}

    public function offsetUnset($offset) {}
}

class Thread extends Threaded implements Countable, Traversable, ArrayAccess
{
    /**
     * @return int
     */
    public function getCreatorId() {}

    /**
     * @return Thread
     */
    public static function getCurrentThread() {}

    /**
     * @return int
     */
    public static function getCurrentThreadId() {}

    /**
     * @return int
     */
    public function getThreadId() {}

    /**
     * @return bool
     */
    public function isJoined() {}

    /**
     * @return bool
     */
    public function isStarted() {}

    /**
     * @return bool
     */
    public function join() {}

    /**
     * @return bool
     */
    public function start(int $options = PTHREADS_INHERIT_ALL) {}
}

class Worker extends Thread implements Traversable, Countable, ArrayAccess
{
    /**
     * @return int
     */
    public function collect(?callable $collector = null) {}

    /**
     * @return int
     */
    public function getStacked() {}

    /**
     * @return bool
     */
    public function isShutdown() {}

    /**
     * @return bool
     */
    public function shutdown() {}

    /**
     * @return int
     */
    public function stack(Threaded $work) {}

    /**
     * @return Threaded|null
     */
    public function unstack() {}
}

interface Collectable
{
    public function isGarbage(): bool;
}

class Volatile extends Threaded implements Collectable, Traversable {}

const PTHREADS_INHERIT_ALL = 1118481;

const PTHREADS_INHERIT_NONE = 0;

const PTHREADS_INHERIT_INI = 1;

const PTHREADS_INHERIT_CONSTANTS = 16;

const PTHREADS_INHERIT_CLASSES = 4096;

const PTHREADS_INHERIT_FUNCTIONS = 256;

const PTHREADS_INHERIT_INCLUDES = 65536;

const PTHREADS_INHERIT_COMMENTS = 1048576;

const PTHREADS_ALLOW_HEADERS = 268435456;
