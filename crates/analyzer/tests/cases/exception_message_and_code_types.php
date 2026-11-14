<?php

declare(strict_types=1);

final class LoudException extends Exception
{
    public function shout(): string
    {
        return strtoupper($this->message) . ($this->code + 1);
    }
}

final class LoudError extends Error
{
    public function shout(): string
    {
        return strtoupper($this->message) . ($this->code + 1);
    }
}

final class DatabaseException extends PDOException
{
    public function sqlState(): int|string
    {
        return $this->code;
    }
}
