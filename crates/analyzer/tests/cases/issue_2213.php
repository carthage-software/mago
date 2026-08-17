<?php

declare(strict_types=1);

class MixedCaseHolder
{
    public const USED = 1;
    public const NEVER = 2;
}

class lowercaseholder
{
    public const USED = 1;
    public const NEVER = 2;
}

class Consumer
{
    public function run(): int
    {
        return MixedCaseHolder::USED + lowercaseholder::USED;
    }
}
