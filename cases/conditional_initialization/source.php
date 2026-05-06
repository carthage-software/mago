<?php

class ConditionalInitBothBranches
{
    public string $name;

    public function __construct(bool $flag)
    {
        if ($flag) {
            $this->name = 'yes';
        } else {
            $this->name = 'no';
        }
    }
}

class ConditionalInitOneBranch
{
    public string $name;

    public function __construct(bool $flag)
    {
        if ($flag) {
            $this->name = 'yes';
        }
    }
}

class ConditionalInitNestedBranches
{
    public string $name;
    public int $age;

    public function __construct(bool $flag1, bool $flag2)
    {
        if ($flag1) {
            if ($flag2) {
                $this->name = 'a';
            } else {
                $this->name = 'b';
            }
            $this->age = 1;
        } else {
            $this->name = 'c';
            $this->age = 2;
        }
    }
}

class ConditionalInitMissingInNestedBranch
{
    public string $name;
    public int $age;

    public function __construct(bool $flag1, bool $flag2)
    {
        if ($flag1) {
            if ($flag2) {
                $this->name = 'a';
                $this->age = 1;
            } else {
                $this->name = 'b';
            }
        } else {
            $this->name = 'c';
            $this->age = 2;
        }
    }
}

class InitWithEarlyReturn
{
    public string $name;

    public function __construct(?string $value)
    {
        if ($value === null) {
            $this->name = 'default';
            return;
        }
        $this->name = $value;
    }
}

class MissingInitWithEarlyReturn
{
    public string $name;

    public function __construct(?string $value)
    {
        if ($value === null) {
            return;
        }
        $this->name = $value;
    }
}

class EarlyReturnInElse
{
    public string $name;

    public function __construct(?string $value)
    {
        if ($value !== null) {
            $this->name = $value;
        } else {
            return;
        }
    }
}

class EarlyReturnAfterInit
{
    public string $name;

    public function __construct(?string $value)
    {
        if ($value === null) {
            $this->name = 'default';
            return;
        }
        $this->name = $value;
    }
}

class ThrowInsteadOfReturn
{
    public string $name;

    public function __construct(?string $value)
    {
        if ($value === null) {
            throw new \InvalidArgumentException('Value required');
        }
        $this->name = $value;
    }
}

class MultiplePropsEarlyReturn
{
    public string $name;
    public int $age;

    public function __construct(?string $name, ?int $age)
    {
        if ($name === null || $age === null) {
            return;
        }
        $this->name = $name;
        $this->age = $age;
    }
}

class PartialInitEarlyReturn
{
    public string $name;
    public int $age;

    public function __construct(?string $name, ?int $age)
    {
        $this->name = $name ?? 'default';
        if ($age === null) {
            return;
        }
        $this->age = $age;
    }
}
