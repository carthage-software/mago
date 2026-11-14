<?php

declare(strict_types=1);

interface Runner
{
    /**
     * @no-named-arguments
     */
    public function run(int $count): void;
}

final class Job implements Runner
{
    public function run(int $times): void
    {
        echo $times;
    }
}
