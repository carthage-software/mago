<?php

declare(strict_types=1);

// Builtin declarations may be tentative: omitting the return type only
// raises a deprecation at runtime, so no error is reported.
class InhMissingRetCountable implements Countable
{
    public function count()
    {
        return 1;
    }
}

// A docblock-only return type on the parent imposes no declaration
// requirement on the child.
abstract class InhMissingRetDocblockBase
{
    /** @return int */
    abstract public function run();
}

class InhMissingRetDocblockChild extends InhMissingRetDocblockBase
{
    public function run()
    {
        return 1;
    }
}
