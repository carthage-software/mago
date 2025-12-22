<?php

declare(strict_types=1);

namespace Quizlet;

/**
 * @template TBacking as string|int
 * @template TValue as TBacking
 */
abstract class AbstractEnum
{
    /** @var TBacking */
    protected $backing;

    /** @var TValue */
    protected $value;
}

/**
 * @extends AbstractEnum<string, 'fizz'|'buzz'>
 */
final class FizzBuzz extends AbstractEnum
{
    const string FIZZ = 'fizz';
    const string BUZZ = 'buzz';
}
