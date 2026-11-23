<?php

declare(strict_types=1);

abstract class LiteralClassStringBase
{
}

class LiteralClassStringA extends LiteralClassStringBase
{
}

function literal_class_string_instantiation_is_safe(): LiteralClassStringBase
{
    $class = LiteralClassStringA::class;

    return new $class();
}
