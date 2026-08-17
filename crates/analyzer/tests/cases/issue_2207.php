<?php

declare(strict_types=1);

class Invokable
{
    public function __invoke(): void {}
}

class NotInvokable {}

/** @param class-string<object&callable> $_class */
function consume(string $_class): void {}

consume(Invokable::class);

// @mago-expect analysis:invalid-argument
consume(NotInvokable::class);
