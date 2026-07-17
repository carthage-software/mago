<?php

declare(strict_types=1);

interface InhMissingRetIface
{
    public function exec(): int;
}

class InhMissingRetImpl implements InhMissingRetIface
{
    /** @mago-expect analysis:incompatible-return-type */
    public function exec()
    {
        return 1;
    }
}

abstract class InhMissingRetBase
{
    abstract public function run(): int;
}

class InhMissingRetChild extends InhMissingRetBase
{
    /** @mago-expect analysis:incompatible-return-type */
    public function run()
    {
        return 1;
    }
}
