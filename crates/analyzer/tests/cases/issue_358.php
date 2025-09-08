<?php

class Thing
{
    /** @var list<Detail> */
    private array $details = [];

    public function addDetail(Detail $detail): void
    {
        $this->details[] = $detail;
    }

    /**
     * @return list<Detail>
     */
    public function getDetails(): array
    {
        return $this->details;
    }
}

class Detail
{
    private Thing $thing;

    public function __construct(Thing $thing)
    {
        $this->thing = $thing;
        $thing->addDetail($this);
    }

    public function getThing(): Thing
    {
        return $this->thing;
    }
}

enum ThingType
{
    case Foo;
    case Bar;
}

class Extractor
{
    public function __invoke(Detail $detail): ThingType
    {
        $details = $detail->getThing()->getDetails();
        $total = count($details);
        $index = (int) array_search($detail, $details, true);

        if (1 === $total) {
            return ThingType::Foo;
        }

        if (2 === $total) {
            return 0 === $index ? ThingType::Foo : ThingType::Bar;
        }

        return 1 === ($index % 2) ? ThingType::Bar : ThingType::Foo;
    }
}
