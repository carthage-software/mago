<?php

declare(strict_types=1);

final class ClassesHasOneConst
{
    public const string FOO = 'foo';
}

/**
 * @mago-expect analysis:non-existent-class-constant
 * @mago-expect analysis:never-return
 */
function classesUndefConst(): mixed
{
    return ClassesHasOneConst::BAR;
}
