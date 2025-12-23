<?php

interface JsonSerializable
{
    public function jsonSerialize(): mixed;
}

interface Stringable
{
    public function __toString(): string;
}

class Message implements Stringable
{
    /**
     * @return 'Hello, World!'
     */
    #[Override]
    public function __toString(): string
    {
        return 'Hello, World!';
    }
}

interface Sendable
{
    public function send(): void;
}

/**
 * @param 'Hello, World!' $message
 */
function greet(string $message): void
{
    echo $message;
}

function process(Sendable&Message $message): void
{
    greet((string) $message);

    $message->send();
}

function process2(Message $message): void
{
    greet((string) $message);
}

/**
 * @template T
 *
 * @param T&Message $message
 *
 * @return T
 */
function process3($message): mixed
{
    greet((string) $message);
    return $message;
}

/**
 * @param JsonSerializable&Stringable $obj
 */
function processObjectIntersection(JsonSerializable&Stringable $obj): string
{
    return (string) $obj;
}
