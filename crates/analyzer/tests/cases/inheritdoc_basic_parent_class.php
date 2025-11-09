<?php declare(strict_types=1);

class ParentClass
{
    /**
     * @param string $name The user's name
     * @param int $age The user's age
     * @return bool True if successful
     */
    public function process($name,  $age)
    {
        return true;
    }

    /**
     * @return array<int, string> List of items
     */
    public function getItems(): array
    {
        return ['item1', 'item2'];
    }
}

class ChildClass extends ParentClass
{
    /** @inheritDoc */
    public function process( $name,  $age)
    {
        // Should inherit: @param string $name, @param int $age, @return bool
        return parent::process($name, $age);
    }

    /** @inheritDoc */
    public function getItems(): array
    {
        $items = parent::getItems();
        foreach ($items as $key => $value) {
            echo $key . ': ' . $value;
        }
        
        return $items;
    }
}

$child = new ChildClass();
$child->process('John', 25);
foreach ($child->getItems() as $item) {
    echo $item . "\n";
}
