<?php

declare(strict_types=1);

class ParentHookBase
{
    public string $name = '' {
        get => $this->name;
        set {
            $this->name = $value;
        }
    }
}

final class ParentHookChild extends ParentHookBase
{
    public string $name {
        get => strtoupper(parent::$name::get());
        set {
            parent::$name::set(trim($value));
        }
    }
}
