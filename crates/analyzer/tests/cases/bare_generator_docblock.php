<?php

declare(strict_types=1);

namespace BareGeneratorDocblock;

use Generator;
use SplObjectStorage;

/** @return Generator */
function bare_return(): Generator
{
    yield 1;
}

/** @param Generator $generator */
function bare_param(Generator $generator): void
{
    foreach ($generator as $_) {
    }
}

/** @return Generator|null */
function bare_nullable(): null|Generator
{
    return null;
}

/** @param list<Generator> $generators */
function bare_nested(array $generators): void
{
    foreach ($generators as $generator) {
        bare_param($generator);
    }
}

/**
 * @template T of Generator
 */
final class Holder
{
    /** @param T $generator */
    public function __construct(
        public Generator $generator,
    ) {}
}

/** @param Holder<Generator> $holder */
function bare_template_argument(Holder $holder): void
{
    bare_param($holder->generator);
}

/** @param SplObjectStorage<object, Generator> $storage */
function bare_storage_value(SplObjectStorage $storage): void
{
    foreach ($storage as $object) {
        bare_param($storage[$object]);
    }
}

bare_param(bare_return());
bare_nullable();
bare_nested([bare_return()]);
bare_template_argument(new Holder(bare_return()));
