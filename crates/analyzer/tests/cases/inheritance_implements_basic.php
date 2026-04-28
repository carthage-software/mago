<?php

declare(strict_types=1);

interface InhBasicService
{
    public function run(): void;
}

class InhBasicWorker implements InhBasicService
{
    public function run(): void
    {
    }
}

(new InhBasicWorker())->run();
