<?php

declare(strict_types=1);

final class EchoAndPrintName
{
    public function __toString(): string
    {
        return 'name';
    }
}

function echo_and_print_accept_stringable_values(EchoAndPrintName $name, Throwable $error, int $count, bool $flag): void
{
    echo $name, $error;
    print $count;
    print $flag;
    print $error;
}
