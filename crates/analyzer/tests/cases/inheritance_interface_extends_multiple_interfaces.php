<?php

declare(strict_types=1);

interface InhA1
{
    public function a(): void;
}

interface InhB1
{
    public function b(): void;
}

interface InhComboIface extends InhA1, InhB1
{
    public function c(): void;
}

class InhCombo implements InhComboIface
{
    public function a(): void
    {
    }

    public function b(): void
    {
    }

    public function c(): void
    {
    }
}

(new InhCombo())->a();
