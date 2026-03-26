<?php

interface BookUrlInfo
{
    public int $id { get; }
    public ?string $slug { get; set; }
}

// Should NOT be inlined (has concrete body)
class Foo
{
    public int $id {
        get => $this->id;
    }
}

// Should NOT be inlined (has modifier)
interface Bar
{
    public int $id {
        protected get;
    }
}

// Empty hook list should remain as-is
class Baz
{
    public int $id {}
}
