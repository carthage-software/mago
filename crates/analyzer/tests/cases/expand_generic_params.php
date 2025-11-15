<?php

enum Color: string
{
    case Red = 'red';
    case Green = 'green';
    case Blue = 'blue';
}

function paint(Color $color): void
{
    print 'Painting in color: ' . $color->value;
}

final readonly class Painter
{
    public function paintAll(): void
    {
        foreach ($this->getFirstGroups() as $group => $colors) {
            $this->paintGroup($group, $colors);
        }

        foreach ($this->getSecondGroups() as $group => $colors) {
            $this->paintGroup($group, $colors);
        }
    }

    /**
     * @param list<Color> $colors
     */
    private function paintGroup(string $group, array $colors): void
    {
        print "Painter is painting group: $group";
        foreach ($colors as $color) {
            paint($color);
        }
    }

    /**
     * @return iterable<string, list<Color::*>>
     */
    private function getFirstGroups(): iterable
    {
        yield 'Warm Colors' => [Color::Red];
        yield 'Cool Colors' => [Color::Blue, Color::Green];
    }

    /**
     * @return Generator<string, list<Color::*>>
     */
    private function getSecondGroups(): Generator
    {
        yield 'Warm Colors' => [Color::Red];
        yield 'Cool Colors' => [Color::Blue, Color::Green];
    }
}
