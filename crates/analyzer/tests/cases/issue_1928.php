<?php

class TestParent
{
    /**
     * @template TKey of key-of<public-properties-of<static>>
     * @param TKey $key
     * @return public-properties-of<static>[TKey]
     */
    public function get(string $key): mixed
    {
        /** @mago-ignore analysis:string-member-selector,mixed-return-statement */
        return $this->{$key};
    }
}

/**
 * @type Member array{
 *     prop1: string,
 *     prop2: int,
 * }
 */
class TestChild extends TestParent
{
    /** @var list<!self::Member> $list */
    public array $list = [
        ['prop1' => 'test', 'prop2' => 1],
        ['prop1' => 'test2', 'prop2' => 2],
    ];

    public int $other = 1;
}

/** @param list<array{prop1: string, prop2: int}> $x */
function expectListOfMember(array $x): void
{
    var_dump($x);
}

/** @param int $x */
function expectInt(int $x): void
{
    var_dump($x);
}

expectListOfMember((new TestChild())->get('list'));
expectInt((new TestChild())->get('other'));

foreach ((new TestChild())->get('list') as $item) {
    expectInt($item['prop2']);
}
