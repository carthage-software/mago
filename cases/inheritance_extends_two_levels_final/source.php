<?php

declare(strict_types=1);

class InhExtTwoLevelGrand {}

final class InhExtTwoLevelMiddle extends InhExtTwoLevelGrand {}

class InhExtTwoLevelChild extends InhExtTwoLevelMiddle {}
