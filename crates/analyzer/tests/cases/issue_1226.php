<?php

declare(strict_types=1);

class Code
{
    public function __construct(
        public string $value {
            set {
                $this->value = strtoupper($value);
            }
        },
    ) {}
}

$e = new Code('example');

echo $e->value;

// @mago-expect analysis:missing-constructor
class BackedSetOnly
{
    public string $name = '';

    public string $upper {
        set {
            $this->upper = strtoupper($value);
            $this->name = $value;
        }
    }
}

$b = new BackedSetOnly();
$b->upper = 'hello';
echo $b->upper;

class VirtualSetOnly
{
    private string $_data = '';

    public string $data {
        set {
            $this->_data = strtoupper($value);
        }
    }

    // @mago-expect analysis:invalid-property-read,never-return
    public function getData(): string
    {
        return $this->data;
    }
}
