<?php

declare(strict_types=1);

/**
 * @throws \DomainException
 */
function flow_throw_in_switch(int $code): string
{
    switch ($code) {
        case 1:
            return 'a';
        case 2:
            return 'b';
        default:
            throw new \DomainException('unknown');
    }
}
