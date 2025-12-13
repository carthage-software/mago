<?php

declare(strict_types=1);

class X
{
    public string $name = 'test';

    /**
     * @var class-string<Test>
     */
    public string $t = Test::class;

    /**
     * @var class-string<Test>
     */
    public static string $t2 = Test::class;
}

class Test
{
    /** @var array<string, mixed> */
    public array $data = [];

    public X $x;

    public function __construct()
    {
        $this->x = new X();
    }

    /** @return non-empty-list<string> */
    public function test(): array
    {
        return ['hi'];
    }

    public static function create(): Test
    {
        return new Test();
    }
}

function get_test(): null|Test
{
    return null;
}

/**
 * @mago-expect analysis:possibly-undefined-array-index
 */
function weird(): void
{
    assert(null === get_test()?->test()[0], description: 'expected null');
    assert(null === get_test()?->data['missing_key'], description: 'expected null');
    assert(null === get_test()?->x->name, description: 'expected null');
    assert(null === get_test()?->x->t::create()->x->name, description: 'expected null');
    assert(null === get_test()?->x->t::create()->test()[0], description: 'expected null');
    assert(null === get_test()?->x::$t2::create()->x->name, description: 'expected null');
    assert(null === get_test()?->x::$t2::create()->test()[0], description: 'expected null');
}
