<?php

declare(strict_types=1);

class Issue2127Parent
{
    public function __construct(
        public readonly string $first,
        public readonly string $second,
    ) {
    }
}

final class Issue2127BadChild extends Issue2127Parent
{
    public function __construct(
        // @mago-expect analysis:invalid-property-write
        public readonly string $first,
        // @mago-expect analysis:invalid-property-write
        public readonly string $second,
        public readonly string $third,
    ) {
        parent::__construct($first, $second);
    }
}

final class Issue2127GoodChild extends Issue2127Parent
{
    public function __construct(
        string $first,
        string $second,
        public readonly string $third,
    ) {
        parent::__construct($first, $second);
    }
}

class Issue2127AssignedParent
{
    public readonly string $value;

    public function __construct()
    {
        $this->value = 'parent';
    }
}

final class Issue2127AssignedBadChild extends Issue2127AssignedParent
{
    public function __construct(
        // @mago-expect analysis:invalid-property-write
        public readonly string $value,
    ) {
        parent::__construct();
    }
}

class Issue2127Grandparent
{
    public function __construct(public readonly string $value)
    {
    }
}

class Issue2127IntermediateParent extends Issue2127Grandparent
{
}

final class Issue2127InheritedConstructorBadChild extends Issue2127IntermediateParent
{
    public function __construct(
        // @mago-expect analysis:invalid-property-write
        public readonly string $value,
    ) {
        parent::__construct($value);
    }
}

class Issue2127PrivateParent
{
    public function __construct(private readonly string $value)
    {
    }

    public function parentValue(): string
    {
        return $this->value;
    }
}

final class Issue2127PrivatePropertyChild extends Issue2127PrivateParent
{
    public function __construct(public readonly string $value)
    {
        parent::__construct($value);
    }
}

readonly class Issue2127ReadonlyParent
{
    public function __construct(
        public string $first,
        public string $second,
    ) {
    }
}

final readonly class Issue2127ReadonlyBadChild extends Issue2127ReadonlyParent
{
    public function __construct(
        // @mago-expect analysis:invalid-property-write
        public string $first,
        // @mago-expect analysis:invalid-property-write
        public string $second,
        public string $third,
    ) {
        parent::__construct($first, $second);
    }
}

final readonly class Issue2127ReadonlyGoodChild extends Issue2127ReadonlyParent
{
    public function __construct(
        string $first,
        string $second,
        public string $third,
    ) {
        parent::__construct($first, $second);
    }
}

new Issue2127BadChild('a', 'b', 'c');
new Issue2127GoodChild('a', 'b', 'c');
new Issue2127AssignedBadChild('a');
new Issue2127InheritedConstructorBadChild('a');
$privatePropertyChild = new Issue2127PrivatePropertyChild('a');
echo $privatePropertyChild->parentValue();
new Issue2127ReadonlyBadChild('a', 'b', 'c');
new Issue2127ReadonlyGoodChild('a', 'b', 'c');
