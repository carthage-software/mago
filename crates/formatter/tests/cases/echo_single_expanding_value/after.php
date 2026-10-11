<?php

function example($value): void
{
    echo sprintf(
        '%s: %s',
        $value->getName(),
        $value->getDescription(),
    );
}
