<?php

function example(string $a, int $b): void
{
    $b = (string) $b;

    echo $a . $b . "\n";
}

example('hello', 123); // ok
example(...['hello', 123]); // ok ( unpacked positional arguments )
example(...['a' => 'hello', 'b' => 123]); // ok ( unpacked named arguments )
example(...['b' => 123, 'a' => 'hello']); // ok ( unpacked named arguments out of order )
example('hello', ...[123]); // ok ( regular argument with unpacked positional arguments )
example('hello', b: 123); // ok ( regular argument with named argument )
example(a: 'hello', b: 123); // ok ( named arguments )
example(b: 123, a: 'hello'); // ok ( named arguments out of order )
example(...['hello'], b: 123); // ok ( unpacked positional arguments with named argument )

example('hello', '123');
example('hello', '123');
example(...['hello', '123']);
example(...['a' => 'hello', 'b' => '123']);
example(...['b' => '123', 'a' => 'hello']);
example('hello', ...['123']);
example('hello', b: '123');
example(a: 'hello', b: '123');
example(b: '123', a: 'hello');
example(...['hello'], b: '123');

example(false, 123);
example(false, 123);
example(...[false, 123]);
example(...['a' => false, 'b' => 123]);
example(...['b' => 123, 'a' => false]);
example(false, ...[123]);
example(false, b: 123);
example(a: false, b: 123);
example(b: 123, a: false);
example(...[false], b: 123);

example(null, 123);
example(null, 123);
example(...[null, 123]);
example(...['a' => null, 'b' => 123]);
example(...['b' => 123, 'a' => null]);
example(null, ...[123]);
example(null, b: 123);
example(a: null, b: 123);
example(b: 123, a: null);
example(...[null], b: 123);

example('hello', false);
example('hello', false);
example(...['hello', false]);
example(...['a' => 'hello', 'b' => false]);
example(...['b' => false, 'a' => 'hello']);
example('hello', ...[false]);
example('hello', b: false);
example(a: 'hello', b: false);
example(b: false, a: 'hello');
example(...['hello'], b: false);

example('hello', null);
example('hello', null);
example(...['hello', null]);
example(...['a' => 'hello', 'b' => null]);
example(...['b' => null, 'a' => 'hello']);
example('hello', ...[null]);
example('hello', b: null);
example(a: 'hello', b: null);
example(b: null, a: 'hello');
example(...['hello'], b: null);

example('hello', 123, 456);
example('hello', 123, 456);
example(...['hello', 123, 456]);
example(...['a' => 'hello', 'b' => 123, 'c' => 456]);
example(...['b' => 123, 'a' => 'hello', 'c' => 456]);
example('hello', ...[123, 456]);
example('hello', 123, b: 456);
example(a: 'hello', b: 123, c: 456);
example(b: 123, a: 'hello', c: 456);
example(...['hello'], b: 123, c: 456);

example();
example(...[]);
example(...['a' => 'hello']);
example(...['b' => 123]);
