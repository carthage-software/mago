<?php

class TestParent
{
    public string $parentProp = 'truc';
}

class TestChild extends TestParent
{
    public int $childProp = 1;

    /** @return public-properties-of<static> */
    public function test(): array
    {
        return [
            'childProp' => $this->childProp,
            'parentProp' => $this->parentProp,
        ];
    }
}

$testchild = new TestChild();

/** @param array{childProp: int, parentProp: string, ...<non-empty-string, mixed>} $arr */
function foo(array $arr): void
{
    var_dump($arr);
}

$v = $testchild->test();
foo($v);
