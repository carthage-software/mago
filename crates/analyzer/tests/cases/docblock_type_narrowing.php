<?php

class A
{
}

class B
{
    public function __construct(
        public A $a,
    ) {}
}

/**
 * @param list{true, A}|list{false, Exception} $data
 *
 * @return Generator<int, A>
 *
 * @throws Exception
 */
function foo(array $data): Generator
{
    [$success, $object_or_exception] = $data;

    if ($success) {
        /** @var A $object_or_exception */
        yield $object_or_exception;
    } else {
        /** @var Exception $object_or_exception */
        throw $object_or_exception;
    }
}
