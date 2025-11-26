<?php

declare(strict_types=1);

final class ParameterType
{
    public const STRING = 2;
}

final class Connection
{
    public const ARRAY_PARAM_OFFSET = 100;
}

final class ArrayParameterType
{
    public const STRING = ParameterType::STRING + Connection::ARRAY_PARAM_OFFSET;
}

function takesIntOrFloat(int|float $value): void
{
}

takesIntOrFloat(ArrayParameterType::STRING);
