<?php

declare(strict_types=1);

/**
 * @template R of int|null
 * @mago-expect analysis:unused-template-parameter
 */
interface QueryInterface
{}

/**
 * @template R of int|null
 * @template Q of QueryInterface<R>
 * @method R handle(Q $query)
 */
interface QueryHandlerInterface
{}

/**
 * @implements QueryInterface<null>
 */
class Foo implements QueryInterface
{}

/**
 * @implements QueryHandlerInterface<null, Foo>
 */
class MyHandler implements QueryHandlerInterface
{
    public function handle(Foo $query): void
    {
        return;
    }
}
