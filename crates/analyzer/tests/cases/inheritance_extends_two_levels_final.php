<?php

declare(strict_types=1);

class InhExtTwoLevelGrand
{
}

final class InhExtTwoLevelMiddle extends InhExtTwoLevelGrand
{
}

/** @mago-expect analysis:extend-final-class */
class InhExtTwoLevelChild extends InhExtTwoLevelMiddle
{
}
