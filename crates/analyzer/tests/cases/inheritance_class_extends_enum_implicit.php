<?php

declare(strict_types=1);

enum InhSimpleEnum
{
    case A;
}

/** @mago-expect analysis:invalid-extend */
final class InhExtendsEnumTwo extends InhSimpleEnum
{
}
