<?php

function foo(string $type, mixed ...$args): void
{
}

foo('test', foo: 'bar');
