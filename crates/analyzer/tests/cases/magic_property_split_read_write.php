<?php

declare(strict_types=1);

namespace MagicPropertySplitReadWrite;

class Value
{
}

/**
 * @property-read Value $forward
 * @property-write Value|string $forward
 * @property-write Value|string $backward
 * @property-read Value $backward
 */
class Container
{
    public function __get(string $_name): mixed
    {
        return new Value();
    }

    public function __set(string $_name, mixed $_value): void
    {
    }
}

function takesValue(Value $_value): void
{
}

$container = new Container();

// Reads produce the @property-read type, whichever tag order declared it.
takesValue($container->forward);
takesValue($container->backward);

// Writes are checked against the @property-write type, not the read type.
$container->forward = new Value();
$container->forward = 'converted';
$container->backward = 'converted';

// A write does not shadow the read type: a read after assigning a write-only-typed value
// still produces the @property-read type (the read goes through __get, not the written value).
takesValue($container->forward);
takesValue($container->backward);

// A value outside the write type is still rejected.
$container->forward = 42; // @mago-expect analysis:invalid-property-assignment-value
