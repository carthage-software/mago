<?php

declare(strict_types=1);

class Redis {}

class CacheEngineRedis
{
    public ?Redis $redis = null;

    public function connect(): void
    {
        if (rand(0, 1)) {
            $this->redis = new Redis();
        } else {
            $this->redis = null;
        }
    }

    public function fetch(string $_k): bool
    {
        if (!$this->redis) {
            $this->connect();
            if (!$this->redis) {
                return false;
            }
        }
        return true;
    }
}

interface Handle
{
    public function tryRead(int $maxBytes): string;

    public function reachedEndOfDataSource(): bool;
}

class StreamProcessor
{
    private ?Handle $currentBody = null;

    public function process(int $maxBytes): string
    {
        if (null === $this->currentBody) {
            return '';
        }

        $chunk = $this->currentBody->tryRead($maxBytes);

        if ($chunk === '' && $this->currentBody->reachedEndOfDataSource()) {
            return '';
        }

        return $chunk;
    }
}

class Suspension
{
    /** @suspends-fiber */
    public function suspend(): void {}
}

class AsyncServer
{
    public bool $closed = false;

    public function process(): void
    {
        if ($this->closed) {
            return;
        }

        while (true) {
            $s = new Suspension();
            $s->suspend();

            if ($this->closed) {
                return;
            }

            break;
        }
    }
}
