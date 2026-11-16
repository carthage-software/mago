<?php

declare(strict_types=1);

class Copyable
{
    final public function __construct() {}

    public function copy(): static
    {
        return new $this();
    }

    public function plain(): static
    {
        return new self(); // @mago-expect analysis:less-specific-return-statement
    }
}

final class FinalCopyable
{
    public function copy(): static
    {
        return new $this();
    }
}
