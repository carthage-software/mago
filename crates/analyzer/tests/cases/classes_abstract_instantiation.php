<?php

declare(strict_types=1);

abstract class ClassesAbstractCannotBeNewed
{
}

/**
 * @mago-expect analysis:abstract-instantiation
 * @mago-expect analysis:impossible-assignment
 */
$_ = new ClassesAbstractCannotBeNewed();
