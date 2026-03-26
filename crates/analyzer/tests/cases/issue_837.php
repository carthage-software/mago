<?php

// @mago-expect analysis:missing-constructor
class Foo
{
    private(set) int $bar {
        get {
            return $this->bar ?? 0;
        }
    }

    public private(set) int $baz {
        get {
            return $this->baz ?? 0;
        }
    }

    protected(set) int $qux {
        get {
            return $this->qux ?? 0;
        }
    }

    public function example(): void
    {
        // Internal writes should be allowed
        $this->bar = 1;
        $this->baz = 1;
        $this->qux = 1;
    }
}

// All should be readable from outside
echo new Foo()->bar;
echo new Foo()->baz;
echo new Foo()->qux;

// Writing should fail
new Foo()->bar = 1; // @mago-expect analysis:invalid-property-write
new Foo()->baz = 1; // @mago-expect analysis:invalid-property-write
new Foo()->qux = 1; // @mago-expect analysis:invalid-property-write
