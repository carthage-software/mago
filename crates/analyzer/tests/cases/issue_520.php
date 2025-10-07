<?php

declare(strict_types=1);

trait InactiveTrait {
    private bool $inactive = false;

    public function setInactive(bool $inactive): void
    {
        $this->inactive = $inactive;
    }
}

class Bar
{
    use InactiveTrait {
        setInactive as private parentSetInactive;
    }

    public function setInactive(bool $inactive): void
    {
        $this->parentSetInactive($inactive);
    }
}

$bar = new Bar();
$bar->setInactive(false);
$bar->parentSetInactive(false); // @mago-expect analysis:invalid-method-access
