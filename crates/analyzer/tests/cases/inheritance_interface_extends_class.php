<?php

declare(strict_types=1);

class InhConcreteClass
{
}

/** @mago-expect analysis:invalid-extend */
interface InhInterfaceExtendsClass extends InhConcreteClass
{
}
