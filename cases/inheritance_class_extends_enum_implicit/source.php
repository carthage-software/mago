<?php

declare(strict_types=1);

enum InhSimpleEnum
{
    case A;
}

final class InhExtendsEnumTwo extends InhSimpleEnum {}
