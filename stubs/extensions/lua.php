<?php

class Lua
{
    /**
     * @var string
     */
    public const LUA_VERSION = '5.1.4';

    public function __construct(null|string $lua_script_file = null) {}

    /**
     * @return $this|null
     */
    public function assign(string $name, $value)
    {
    }

    public function call(callable $lua_func, array $args = [], bool $use_self = false)
    {
    }

    public function eval(string $statements)
    {
    }

    public function include(string $file)
    {
    }

    public function getVersion(): string
    {
    }

    /**
     * @return $this|null|false
     */
    public function registerCallback(string $name, callable $function)
    {
    }
}
