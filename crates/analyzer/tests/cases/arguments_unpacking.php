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

example('hello', '123'); // @mago-expect analysis:invalid-argument
example('hello', '123'); // @mago-expect analysis:invalid-argument
example(...['hello', '123']); // @mago-expect analysis:invalid-argument
example(...['a' => 'hello', 'b' => '123']); // @mago-expect analysis:invalid-argument
example(...['b' => '123', 'a' => 'hello']); // @mago-expect analysis:invalid-argument
example('hello', ...['123']); // @mago-expect analysis:invalid-argument
example('hello', b: '123'); // @mago-expect analysis:invalid-argument
example(a: 'hello', b: '123'); // @mago-expect analysis:invalid-argument
example(b: '123', a: 'hello'); // @mago-expect analysis:invalid-argument
example(...['hello'], b: '123'); // @mago-expect analysis:invalid-argument

example(false, 123); // @mago-expect analysis:false-argument
example(false, 123); // @mago-expect analysis:false-argument
example(...[false, 123]); // @mago-expect analysis:false-argument
example(...['a' => false, 'b' => 123]); // @mago-expect analysis:false-argument
example(...['b' => 123, 'a' => false]); // @mago-expect analysis:false-argument
example(false, ...[123]); // @mago-expect analysis:false-argument
example(false, b: 123); // @mago-expect analysis:false-argument
example(a: false, b: 123); // @mago-expect analysis:false-argument
example(b: 123, a: false); // @mago-expect analysis:false-argument
example(...[false], b: 123); // @mago-expect analysis:false-argument

example(null, 123); // @mago-expect analysis:null-argument
example(null, 123); // @mago-expect analysis:null-argument
example(...[null, 123]); // @mago-expect analysis:null-argument
example(...['a' => null, 'b' => 123]); // @mago-expect analysis:null-argument
example(...['b' => 123, 'a' => null]); // @mago-expect analysis:null-argument
example(null, ...[123]); // @mago-expect analysis:null-argument
example(null, b: 123); // @mago-expect analysis:null-argument
example(a: null, b: 123); // @mago-expect analysis:null-argument
example(b: 123, a: null); // @mago-expect analysis:null-argument
example(...[null], b: 123); // @mago-expect analysis:null-argument

example('hello', false); // @mago-expect analysis:false-argument
example('hello', false); // @mago-expect analysis:false-argument
example(...['hello', false]); // @mago-expect analysis:false-argument
example(...['a' => 'hello', 'b' => false]); // @mago-expect analysis:false-argument
example(...['b' => false, 'a' => 'hello']); // @mago-expect analysis:false-argument
example('hello', ...[false]); // @mago-expect analysis:false-argument
example('hello', b: false); // @mago-expect analysis:false-argument
example(a: 'hello', b: false); // @mago-expect analysis:false-argument
example(b: false, a: 'hello'); // @mago-expect analysis:false-argument
example(...['hello'], b: false); // @mago-expect analysis:false-argument

example('hello', null); // @mago-expect analysis:null-argument
example('hello', null); // @mago-expect analysis:null-argument
example(...['hello', null]); // @mago-expect analysis:null-argument
example(...['a' => 'hello', 'b' => null]); // @mago-expect analysis:null-argument
example(...['b' => null, 'a' => 'hello']); // @mago-expect analysis:null-argument
example('hello', ...[null]); // @mago-expect analysis:null-argument
example('hello', b: null); // @mago-expect analysis:null-argument
example(a: 'hello', b: null); // @mago-expect analysis:null-argument
example(b: null, a: 'hello'); // @mago-expect analysis:null-argument
example(...['hello'], b: null); // @mago-expect analysis:null-argument

example('hello', 123, 456); // @mago-expect analysis:too-many-arguments
example('hello', 123, 456); // @mago-expect analysis:too-many-arguments
example(...['hello', 123, 456]); // @mago-expect analysis:too-many-arguments
example(...['a' => 'hello', 'b' => 123, 'c' => 456]); // @mago-expect analysis:too-many-arguments,invalid-named-argument
example(...['b' => 123, 'a' => 'hello', 'c' => 456]); // @mago-expect analysis:too-many-arguments,invalid-named-argument
example('hello', ...[123, 456]); // @mago-expect analysis:too-many-arguments
example('hello', 123, b: 456); // @mago-expect analysis:too-many-arguments,named-argument-overrides-positional
example(a: 'hello', b: 123, c: 456); // @mago-expect analysis:too-many-arguments,invalid-named-argument
example(b: 123, a: 'hello', c: 456); // @mago-expect analysis:too-many-arguments,invalid-named-argument
example(...['hello'], b: 123, c: 456); // @mago-expect analysis:too-many-arguments,invalid-named-argument

example(); // @mago-expect analysis:too-few-arguments
example(...[]); // @mago-expect analysis:too-few-arguments
example(...['a' => 'hello']); // @mago-expect analysis:too-few-arguments
example(...['b' => 123]); // @mago-expect analysis:too-few-arguments
