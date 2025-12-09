<?php

class VirtualGetOnly
{
    public string $computed {
        get => 'computed value';
    }

    public function __construct() {}
}

class VirtualGetSet
{
    private string $first = '';
    private string $last = '';

    public string $fullName {
        get => $this->first . ' ' . $this->last;
        set {
            $parts = explode(' ', $value, 2);
            $this->first = $parts[0];
            $this->last = $parts[1] ?? '';
        }
    }

    public function __construct() {}
}

class VirtualReadOnly
{
    private int $value = 42;

    public int $doubled {
        get => $this->value * 2;
    }

    public function __construct() {}
}

class MultipleVirtual
{
    private string $data = '';

    public string $upper {
        get => strtoupper($this->data);
    }

    public string $lower {
        get => strtolower($this->data);
    }

    public int $length {
        get => strlen($this->data);
    }

    public function __construct() {}
}

class PromotedWithHook
{
    public function __construct(
        public string $name {
            get => strtoupper($this->name);
        },
    ) {}
}

class MixedPromotedVirtual
{
    public function __construct(
        public string $firstName,
        public string $lastName,
    ) {}

    public string $fullName {
        get => $this->firstName . ' ' . $this->lastName;
    }
}

interface HasHookedProperty
{
    public string $value {
        get;
        set;
    }
}

class ImplementsHookedProperty implements HasHookedProperty
{
    private string $_value = '';

    public string $value {
        get => $this->_value;
        set => $this->_value = $value;
    }
}

class MixedRegularAndVirtual
{
    public string $name = 'default';

    public string $upperName {
        get => strtoupper($this->name);
    }

    public function __construct() {}
}

class InitializedPlusVirtual
{
    public string $name;

    public string $greeting {
        get => 'Hello, ' . $this->name;
    }

    public function __construct()
    {
        $this->name = 'World';
    }
}

final class FinalWithHooks
{
    private int $count = 0;

    public int $value {
        get => $this->count;
        set => $this->count = max(0, $value);
    }

    public function __construct() {}
}

abstract class AbstractWithVirtualProperty
{
    public string $computed {
        get => 'abstract computed';
    }
}

class ConcreteWithVirtualProperty extends AbstractWithVirtualProperty
{
    public function __construct() {}
}
