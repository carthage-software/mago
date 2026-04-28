<?php

declare(strict_types=1);

final class ClassesClassKeywordOnObject
{
}

$obj = new ClassesClassKeywordOnObject();
echo $obj::class;
