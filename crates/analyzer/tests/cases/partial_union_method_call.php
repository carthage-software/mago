<?php

declare(strict_types=1);

final class Expectation
{
    public function once(): self
    {
        return $this;
    }

    public function andReturn(int $value): int
    {
        return $value;
    }
}

final class Double
{
    public function expect(string $_method): Expectation
    {
        return new Expectation();
    }
}

final class Service
{
    public function run(): int
    {
        return 1;
    }
}

function configure(Service|Double $double): int
{
    return $double->expect('run')->once()->andReturn(1); // @mago-expect analysis:non-existent-method
}

function takeInt(int $x): int
{
    return $x;
}

function configureArg(Service|Double $double): int
{
    return takeInt($double->expect('run')->once()->andReturn(1)); // @mago-expect analysis:non-existent-method
}
