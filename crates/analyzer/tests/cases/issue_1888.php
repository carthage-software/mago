<?php

declare(strict_types=1);

namespace App;

interface A {}

interface B {}

/** @return list{A, B} */
function test(A $module): array
{
    /** @var B $module🤔_lorem_ipsum_dolor_sit_amet_consete */
    $module🤔_lorem_ipsum_dolor_sit_amet_consete = new class implements B {};

    return [$module, $module🤔_lorem_ipsum_dolor_sit_amet_consete];
}
