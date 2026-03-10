<?php

declare(strict_types=1);

interface HandlerInterface1346 {}

final class Pipeline1346
{
    /**
     * @var WeakMap<HandlerInterface1346, HandlerInterface1346>
     */
    private WeakMap $cache;

    public function __construct()
    {
        $this->cache = new WeakMap();
    }

    public function get(HandlerInterface1346 $h): HandlerInterface1346
    {
        return $this->cache[$h] ?? $h;
    }

    public function set(HandlerInterface1346 $h, HandlerInterface1346 $c): void
    {
        $this->cache[$h] = $c;
    }
}
