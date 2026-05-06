<?php

declare(strict_types=1);

final class ClassesHasOneConst
{
    public const string FOO = 'foo';
}

/**
 */
function classesUndefConst(): mixed
{
    return ClassesHasOneConst::BAR;
}
