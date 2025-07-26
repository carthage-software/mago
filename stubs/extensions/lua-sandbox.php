<?php

class LuaSandbox
{
    public const SAMPLES = 0;
    public const SECONDS = 1;
    public const PERCENT = 2;

    /**
     * @param string $name
     *
     * @return array|bool
     */
    public function callFunction($name, array $arguments)
    {
    }

    public function disableProfiler()
    {
    }

    /**
     * @param float $period
     *
     * @return bool
     */
    public function enableProfiler($period = 0.02)
    {
    }

    /**
     * @return float
     */
    public function getCPUUsage()
    {
    }

    /**
     * @return int
     */
    public function getMemoryUsage()
    {
    }

    /**
     * @return int
     */
    public function getPeakMemoryUsage()
    {
    }

    /**
     * @param int $units
     *
     * @return array
     */
    public function getProfilerFunctionReport($units = LuaSandbox::SECONDS)
    {
    }

    /**
     * @return array
     */
    public static function getVersionInfo()
    {
    }

    /**
     * @param string $code
     * @param string $chunkName
     *
     * @return LuaSandboxFunction
     */
    public function loadBinary($code, $chunkName = '')
    {
    }

    /**
     * @param string $code
     * @param string $chunkName
     *
     * @return LuaSandboxFunction
     */
    public function loadString($code, $chunkName = '')
    {
    }

    /**
     * @return bool
     */
    public function pauseUsageTimer()
    {
    }

    /**
     * @param string $libname
     * @param array $functions
     */
    public function registerLibrary($libname, $functions)
    {
    }

    /**
     * @param bool|float $limit
     */
    public function setCPULimit($limit)
    {
    }

    /**
     * @param int $limit
     *
     * @throws LuaSandboxMemoryError
     */
    public function setMemoryLimit($limit)
    {
    }

    public function unpauseUsageTimer()
    {
    }

    /**
     * @param callable $function
     *
     * @return LuaSandboxFunction
     */
    public function wrapPhpFunction($function)
    {
    }
}

class LuaSandboxFunction
{
    /**
     * @return array|bool
     */
    public function call(string ...$args)
    {
    }

    /**
     * @return string
     */
    public function dump()
    {
    }
}

class LuaSandboxError extends Exception
{
    public const RUN = 2;
    public const SYNTAX = 3;
    public const MEM = 4;
    public const ERR = 5;
}

class LuaSandboxRuntimeError extends LuaSandboxError
{
}

class LuaSandboxFatalError extends LuaSandboxError
{
}

class LuaSandboxErrorError extends LuaSandboxFatalError
{
}

class LuaSandboxMemoryError extends LuaSandboxFatalError
{
}

class LuaSandboxSyntaxError extends LuaSandboxFatalError
{
}

class LuaSandboxTimeoutError extends LuaSandboxFatalError
{
}
