<?php

class SyncMutex
{
    /**
     * @param string $name
     *
     * @throws Exception
     */
    public function __construct($name = null) {}

    /**
     * @param int $wait
     *
     * @return bool
     */
    public function lock($wait = -1)
    {
    }

    /**
     * @param bool $all
     *
     * @return bool
     */
    public function unlock($all = false)
    {
    }
}

class SyncSemaphore
{
    /**
     * @param string $name
     * @param int    $initialval
     * @param bool   $autounlock
     *
     * @throws Exception
     */
    public function __construct($name, $initialval = 1, $autounlock = true) {}

    /**
     * @param int $wait
     *
     * @return bool
     */
    public function lock($wait = -1)
    {
    }

    /**
     * @param-out int $prevcount
     *
     * @return bool
     */
    public function unlock(&$prevcount = 0)
    {
    }
}

class SyncEvent
{
    /**
     * @param string $name
     * @param bool   $manual
     * @param bool   $prefire
     *
     * @throws Exception
     */
    public function __construct(string $name, bool $manual = false, bool $prefire = false) {}

    /**
     * @return bool
     */
    public function fire()
    {
    }

    /**
     * @return bool
     */
    public function reset()
    {
    }

    /**
     * @param int $wait
     *
     * @return bool
     */
    public function wait($wait = -1)
    {
    }
}

class SyncReaderWriter
{
    /**
     * @param string $name
     * @param bool   $autounlock
     *
     * @throws Exception
     */
    public function __construct($name, $autounlock = true) {}

    /**
     * @param int $wait
     *
     * @return bool
     */
    public function readlock($wait = -1)
    {
    }

    /**
     * @return bool
     */
    public function readunlock()
    {
    }

    /**
     * @param int $wait
     *
     * @return bool
     */
    public function writelock($wait = -1)
    {
    }

    /**
     * @return bool
     */
    public function writeunlock()
    {
    }
}

class SyncSharedMemory
{
    /**
     * @param string $name
     * @param int    $size
     *
     * @throws Exception
     */
    public function __construct($name, $size) {}

    /**
     * @return bool
     */
    public function first()
    {
    }

    /**
     * @param int $start
     * @param int $length
     *
     * @return string
     */
    public function read($start = 0, $length)
    {
    }

    /**
     * @return int
     */
    public function size()
    {
    }

    /**
     * @param string $string
     * @param int    $start
     *
     * @return int
     */
    public function write($string, $start = 0)
    {
    }
}
