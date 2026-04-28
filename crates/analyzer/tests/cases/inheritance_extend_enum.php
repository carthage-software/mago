<?php

declare(strict_types=1);

enum InhSomeEnum
{
    case A;
}

/** @mago-expect analysis:invalid-extend */
class InhExtendsEnum extends InhSomeEnum
{
}
