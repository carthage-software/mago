<?php

declare(strict_types=1);

class InhPlainClass
{
}

/** @mago-expect analysis:invalid-implement */
class InhBadImplements implements InhPlainClass
{
}
