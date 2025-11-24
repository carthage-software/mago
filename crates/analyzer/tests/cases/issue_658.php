<?php

/** @param "A"|"B" $l */
function onLetter(string $l): void
{
}

function onText(string $t): void
{
    if (in_array($t, ['A', 'B'], true)) {
        onLetter($t);
    }
}
